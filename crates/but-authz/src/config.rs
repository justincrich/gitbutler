use std::{borrow::Borrow, collections::BTreeMap, path::Path, str};

use anyhow::{Context, anyhow};
use serde::{Deserialize, Serialize};

use crate::{AuthoritySet, DenialClass, Group, GroupName, PrincipalId};

const PERMISSIONS_PATH: &str = ".gitbutler/permissions.toml";
const GATES_PATH: &str = ".gitbutler/gates.toml";
const CONFIG_INVALID: &str = "config.invalid";

/// Return the repository-relative governance permissions path.
///
/// ```
/// assert_eq!(but_authz::permissions_path(), ".gitbutler/permissions.toml");
/// ```
pub fn permissions_path() -> &'static str {
    PERMISSIONS_PATH
}

/// Return the repository-relative governance gates path.
///
/// ```
/// assert_eq!(but_authz::gates_path(), ".gitbutler/gates.toml");
/// ```
pub fn gates_path() -> &'static str {
    GATES_PATH
}

/// Load committed governance config from the supplied target ref.
///
/// The loader reads `.gitbutler/permissions.toml` and `.gitbutler/gates.toml`
/// from the target ref's tree through `gix`; it never consults the working tree.
///
/// ```
/// # fn example(repo: &gix::Repository) -> Result<(), but_authz::ConfigError> {
/// let config = but_authz::load_governance_config(repo, "refs/heads/main")?;
/// assert!(config.principals().is_empty() || !config.branches().is_empty());
/// # Ok(())
/// # }
/// ```
pub fn load_governance_config(
    repo: &gix::Repository,
    target_ref: &str,
) -> Result<GovConfig, ConfigError> {
    load_governance_config_inner(repo, target_ref).map_err(ConfigError::invalid)
}

/// Load the raw `[[principal]]`/`[[group]]` wire format from the committed
/// `.gitbutler/permissions.toml` at the supplied target ref.
///
/// Unlike [`load_governance_config`] (which desugars roles into a flat
/// `GovConfig` authority map and discards non-enforcement fields), this
/// preserves the additive optional fields that do not flow into enforcement —
/// notably [`PrincipalWire::kind`], the descriptor the agent-PR tag derivation
/// reads. Reads at the target ref like all governance config (anti-self-
/// escalation); the working tree is never consulted. Returns `Ok(PermissionsWire::default())`
/// when the target ref does not carry a committed `permissions.toml` (a
/// non-governed ref has no tag to derive).
///
/// ```
/// # fn example(repo: &gix::Repository) -> anyhow::Result<()> {
/// let wire = but_authz::load_permissions_wire(repo, "refs/heads/main")?;
/// assert!(wire.principal.is_empty() || !wire.principal.is_empty());
/// # Ok(())
/// # }
/// ```
pub fn load_permissions_wire(
    repo: &gix::Repository,
    target_ref: &str,
) -> anyhow::Result<PermissionsWire> {
    let permissions_blob = match read_config_blob(repo, target_ref, PERMISSIONS_PATH) {
        Ok(blob) => blob,
        Err(error)
            if error
                .to_string()
                .starts_with(&format!("missing {PERMISSIONS_PATH} ")) =>
        {
            return Ok(PermissionsWire::default());
        }
        Err(error) => return Err(error),
    };
    let permissions = toml::from_str::<PermissionsWire>(&permissions_blob)
        .with_context(|| format!("parsing {PERMISSIONS_PATH} at {target_ref}"))?;
    Ok(permissions)
}

/// Whether governance is opted-in at `target_ref`.
///
/// Governance is **opt-in by presence**: a ref is governed once it commits at
/// least one of `.gitbutler/permissions.toml` / `.gitbutler/gates.toml` into its
/// tree. This is the gate's discriminator — `false` means the ref is ungoverned
/// and the gate does not run; `true` means the gate must
/// [`load_governance_config`], which fails closed `config.invalid` if a
/// companion file is missing (incomplete governance) or malformed. The working
/// tree is never consulted; an unresolvable ref/commit/tree is treated as
/// governed so the loader classifies the fault rather than silently allowing.
///
/// This is the single source of truth for the governance file paths — callers
/// must not re-derive `.gitbutler/*.toml` literals.
pub fn governance_present(repo: &gix::Repository, target_ref: &str) -> anyhow::Result<bool> {
    let mut reference = match repo.find_reference(target_ref) {
        Ok(reference) => reference,
        Err(_) => return Ok(true),
    };
    let commit = match reference.peel_to_commit() {
        Ok(commit) => commit,
        Err(_) => return Ok(true),
    };
    let tree = match commit.tree() {
        Ok(tree) => tree,
        Err(_) => return Ok(true),
    };

    Ok(tree_has_path(&tree, PERMISSIONS_PATH)? || tree_has_path(&tree, GATES_PATH)?)
}

fn tree_has_path(tree: &gix::Tree<'_>, path: &str) -> anyhow::Result<bool> {
    Ok(tree.lookup_entry_by_path(Path::new(path))?.is_some())
}

/// Governance config normalized for authorization checks.
///
/// Role entries are desugared during load, so consumers only see flat
/// [`AuthoritySet`] values.
///
/// ```
/// let config = but_authz::GovConfig::new([], [], []);
/// assert!(config.principals().is_empty());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovConfig {
    principals: BTreeMap<PrincipalId, AuthoritySet>,
    groups: BTreeMap<GroupName, Group>,
    branches: BTreeMap<BranchName, BranchProtection>,
}

impl GovConfig {
    /// Create a governance config from normalized maps.
    ///
    /// ```
    /// let config = but_authz::GovConfig::new([], [], []);
    /// assert!(config.branches().is_empty());
    /// ```
    pub fn new<P, G, B>(principals: P, groups: G, branches: B) -> Self
    where
        P: IntoIterator<Item = (PrincipalId, AuthoritySet)>,
        G: IntoIterator<Item = (GroupName, Group)>,
        B: IntoIterator<Item = (BranchName, BranchProtection)>,
    {
        Self {
            principals: principals.into_iter().collect(),
            groups: groups.into_iter().collect(),
            branches: branches.into_iter().collect(),
        }
    }

    /// Return loaded effective authority sets by principal id.
    ///
    /// ```
    /// let config = but_authz::GovConfig::new([], [], []);
    /// assert!(config.principals().is_empty());
    /// ```
    pub fn principals(&self) -> &BTreeMap<PrincipalId, AuthoritySet> {
        &self.principals
    }

    /// Return the effective authority set for one principal.
    ///
    /// ```
    /// let config = but_authz::GovConfig::new([], [], []);
    /// assert!(config.principal_authorities(&but_authz::PrincipalId::new("dev")).is_none());
    /// ```
    pub fn principal_authorities(&self, principal_id: &PrincipalId) -> Option<&AuthoritySet> {
        self.principals.get(principal_id)
    }

    /// Return loaded groups by group name.
    ///
    /// ```
    /// let config = but_authz::GovConfig::new([], [], []);
    /// assert!(config.groups().is_empty());
    /// ```
    pub fn groups(&self) -> &BTreeMap<GroupName, Group> {
        &self.groups
    }

    /// Return loaded branch protection records by branch name.
    ///
    /// ```
    /// let config = but_authz::GovConfig::new([], [], []);
    /// assert!(config.branches().is_empty());
    /// ```
    pub fn branches(&self) -> &BTreeMap<BranchName, BranchProtection> {
        &self.branches
    }

    /// Return the protection record for a branch.
    ///
    /// ```
    /// let config = but_authz::GovConfig::new([], [], []);
    /// assert!(config.branch("main").is_none());
    /// ```
    pub fn branch(&self, branch_name: &str) -> Option<&BranchProtection> {
        self.branches.get(branch_name)
    }
}

/// Stable branch name used by governance gates.
///
/// ```
/// let name = but_authz::BranchName::new("main");
/// assert_eq!(name.as_str(), "main");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BranchName(String);

impl BranchName {
    /// Create a branch name.
    ///
    /// ```
    /// let name = but_authz::BranchName::new("main");
    /// assert_eq!(name.as_str(), "main");
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Return the branch name as a string slice.
    ///
    /// ```
    /// assert_eq!(but_authz::BranchName::new("main").as_str(), "main");
    /// ```
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for BranchName {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

/// Protection settings for one governed branch.
///
/// ```
/// let protection = but_authz::BranchProtection::new(true);
/// assert!(protection.protected());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BranchProtection {
    protected: bool,
}

impl BranchProtection {
    /// Create branch protection settings.
    ///
    /// ```
    /// let protection = but_authz::BranchProtection::new(false);
    /// assert!(!protection.protected());
    /// ```
    pub fn new(protected: bool) -> Self {
        Self { protected }
    }

    /// Return whether this branch is protected.
    ///
    /// ```
    /// assert!(but_authz::BranchProtection::new(true).protected());
    /// ```
    pub fn protected(self) -> bool {
        self.protected
    }
}

/// Typed governance config load failure.
///
/// Consumers can use [`ConfigError::code`] for stable classification without
/// matching display strings.
///
/// ```
/// # fn handle(error: but_authz::ConfigError) {
/// assert_eq!(error.code(), "config.invalid");
/// # }
/// ```
#[derive(Debug, thiserror::Error)]
#[error("invalid governance config: {message}")]
pub struct ConfigError {
    message: String,
    #[source]
    source: anyhow::Error,
    /// Steering classification — who can recover. `None` until STEER-004
    /// populates the value at the config-load denial sites; `to_envelope`
    /// and the custom [`Serialize`] impl emit `class` only when `Some`.
    pub class: Option<DenialClass>,
    /// Optional "do not" hint — verbs the actor must NOT attempt. Emitted
    /// only when `Some`; `None` omits the key entirely (matches
    /// `Option::is_none` skip semantics used by the other carriers).
    pub do_not: Option<&'static str>,
}

impl ConfigError {
    /// Return the stable consumer-facing error classification.
    ///
    /// ```
    /// # fn handle(error: but_authz::ConfigError) {
    /// assert_eq!(error.code(), "config.invalid");
    /// # }
    /// ```
    pub fn code(&self) -> &'static str {
        CONFIG_INVALID
    }

    fn invalid(source: anyhow::Error) -> Self {
        Self {
            message: source.to_string(),
            source,
            class: Some(DenialClass::OperatorRequired),
            do_not: Some("do not retry — an operator must fix the committed .gitbutler config"),
        }
    }
}

impl Serialize for ConfigError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct as _;

        let mut field_count = 2; // code + message are always present
        if self.class.is_some() {
            field_count += 1;
        }
        if self.do_not.is_some() {
            field_count += 1;
        }
        let mut state = serializer.serialize_struct("ConfigError", field_count)?;
        state.serialize_field("code", &self.code())?;
        state.serialize_field("message", &self.message)?;
        if let Some(class) = self.class {
            state.serialize_field("class", &class)?;
        }
        if let Some(do_not) = self.do_not {
            state.serialize_field("do_not", do_not)?;
        }
        state.end()
    }
}

fn load_governance_config_inner(
    repo: &gix::Repository,
    target_ref: &str,
) -> anyhow::Result<GovConfig> {
    let permissions_blob = read_config_blob(repo, target_ref, PERMISSIONS_PATH)?;
    let gates_blob = read_config_blob(repo, target_ref, GATES_PATH)?;

    let permissions = toml::from_str::<PermissionsWire>(&permissions_blob)
        .with_context(|| format!("parsing {PERMISSIONS_PATH} at {target_ref}"))?;
    let gates = toml::from_str::<GatesWire>(&gates_blob)
        .with_context(|| format!("parsing {GATES_PATH} at {target_ref}"))?;

    let (principals, groups) = normalize_permissions(permissions)?;
    let branches = normalize_gates(gates)?;

    Ok(GovConfig::new(principals, groups, branches))
}

fn read_config_blob(
    repo: &gix::Repository,
    target_ref: &str,
    path: &'static str,
) -> anyhow::Result<String> {
    let mut reference = repo
        .find_reference(target_ref)
        .with_context(|| format!("resolving target ref {target_ref}"))?;
    let commit = reference
        .peel_to_commit()
        .with_context(|| format!("peeling {target_ref} to a commit"))?;
    let tree = commit
        .tree()
        .with_context(|| format!("reading tree for {target_ref}"))?;
    let entry = tree
        .lookup_entry_by_path(Path::new(path))
        .with_context(|| format!("looking up {path} in {target_ref}"))?
        .ok_or_else(|| anyhow!("missing {path} at {target_ref}"))?;
    let blob = repo
        .find_blob(entry.id())
        .with_context(|| format!("reading {path} blob at {target_ref}"))?;
    let content = str::from_utf8(&blob.data)
        .with_context(|| format!("decoding {path} at {target_ref} as UTF-8"))?;

    Ok(content.to_owned())
}

fn normalize_permissions(
    permissions: PermissionsWire,
) -> anyhow::Result<(
    BTreeMap<PrincipalId, AuthoritySet>,
    BTreeMap<GroupName, Group>,
)> {
    let mut groups = BTreeMap::new();
    let mut group_authorities = BTreeMap::new();

    for group in &permissions.group {
        let name = GroupName::new(group.name.clone());
        let authorities = authority_set_from_wire(
            &group.permissions,
            group.role.as_deref(),
            &format!("group {}", group.name),
        )?;
        let members = group.members.iter().cloned().map(PrincipalId::new);
        let parsed = Group::new(name.clone(), authorities.clone(), members);

        if groups.insert(name.clone(), parsed).is_some() {
            return Err(anyhow!("duplicate group {}", group.name));
        }
        group_authorities.insert(name, authorities);
    }

    let mut principals = BTreeMap::new();
    // This load-time fold is the single source of truth for effective authority:
    // direct grants plus groups named by a principal and groups naming a member.
    for principal in &permissions.principal {
        let mut authorities = authority_set_from_wire(
            &principal.permissions,
            principal.role.as_deref(),
            &format!("principal {}", principal.id),
        )?;

        for group_name in &principal.groups {
            let group_key = GroupName::new(group_name.clone());
            let group_set = group_authorities.get(&group_key).ok_or_else(|| {
                anyhow!(
                    "principal {} references undefined group {group_name}",
                    principal.id
                )
            })?;
            authorities = authorities.union(group_set);
        }

        let id = PrincipalId::new(principal.id.clone());
        if principals.insert(id, authorities).is_some() {
            return Err(anyhow!("duplicate principal {}", principal.id));
        }
    }

    for group in &permissions.group {
        let group_key = GroupName::new(group.name.clone());
        let group_set = group_authorities
            .get(&group_key)
            .ok_or_else(|| anyhow!("group {} was not normalized", group.name))?;

        for member in &group.members {
            let id = PrincipalId::new(member.clone());
            let authorities = principals
                .get(&id)
                .map_or_else(|| group_set.clone(), |existing| existing.union(group_set));
            principals.insert(id, authorities);
        }
    }

    Ok((principals, groups))
}

fn normalize_gates(gates: GatesWire) -> anyhow::Result<BTreeMap<BranchName, BranchProtection>> {
    let mut branches = BTreeMap::new();
    for branch in gates.branch {
        let name = BranchName::new(branch.name.clone());
        if branches
            .insert(name, BranchProtection::new(branch.protected))
            .is_some()
        {
            return Err(anyhow!("duplicate branch {}", branch.name));
        }
    }
    Ok(branches)
}

fn authority_set_from_wire(
    permissions: &[String],
    role: Option<&str>,
    subject: &str,
) -> anyhow::Result<AuthoritySet> {
    let listed = AuthoritySet::parse(permissions.iter().map(String::as_str))
        .with_context(|| format!("parsing authority list for {subject}"))?;
    let role_set = AuthoritySet::from_optional_role(role)
        .with_context(|| format!("desugaring authority role for {subject}"))?;

    Ok(listed.union(&role_set))
}

/// Raw permissions TOML wire format.
///
/// This preserves role sugar and unrelated entries for read-modify-write callers.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PermissionsWire {
    #[serde(default)]
    pub principal: Vec<PrincipalWire>,
    #[serde(default)]
    pub group: Vec<GroupWire>,
}

/// Raw `[[principal]]` permissions TOML entry.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrincipalWire {
    pub id: String,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Additive, enforcement-neutral descriptor naming the principal's kind
    /// (e.g. `"agent"` / `"human"`). Source-of-truth for the agent-PR tag
    /// derivation, which reads this at the target ref like all governance
    /// config. Older committed files without `kind` deserialize to `None`
    /// (default-human). The `kind` field does NOT enter `GovConfig.principals`
    /// and no gate reads it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default)]
    pub groups: Vec<String>,
}

/// Raw `[[group]]` permissions TOML entry.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GroupWire {
    pub name: String,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default)]
    pub members: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GatesWire {
    #[serde(default)]
    branch: Vec<BranchWire>,
    #[allow(dead_code)]
    #[serde(default)]
    gate: Vec<GateWire>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BranchWire {
    name: String,
    protected: bool,
}

/// Raw `[[gate]]` gates TOML entry.
///
/// Mirrors `but_api::legacy::merge_gate::GateWire` field-for-field so the
/// loader accepts the full review-requirement array without `config.invalid`.
/// The normalization layer still ignores gate entries; the writer task
/// (MGMT-BE-004A) owns lossless round-trip serialization.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct GateWire {
    branch: String,
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    min_approvals: usize,
    #[serde(default)]
    require_approval_from_group: Vec<String>,
    #[serde(default)]
    require_distinct_from_author: bool,
}
