use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context as _, anyhow};
use but_authz::{
    Authority, AuthoritySet, Denial, PermissionsWire, PrincipalId, PrincipalWire,
    load_governance_config, permissions_path,
};
use serde::Serialize;

use super::config_mutate::enforce_administration_write_gate;

/// Operator-facing caveat for working-tree governance writes.
pub const REF_PIN_CAVEAT: &str = "takes effect once committed to the target branch";

/// Result of a governance permission write.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PermWriteOutcome {
    /// Principal whose direct permissions were changed or inspected.
    pub principal: String,
    /// Parsed authority tokens supplied by the caller.
    pub authorities: Vec<String>,
    /// Ref-pin caveat for the operator.
    pub caveat: &'static str,
}

/// Listed authority for one principal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PermListEntry {
    /// Functional authority token.
    pub authority: String,
    /// Literal marker for a working-tree grant not committed at the target ref.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<&'static str>,
}

/// Result of listing one principal's permissions.
#[derive(Clone, PartialEq, Eq, Serialize)]
pub struct PermListOutcome {
    /// Principal whose permissions were listed.
    pub principal: String,
    /// Committed authorities plus pending working-tree direct grants.
    pub authorities: Vec<PermListEntry>,
}

impl std::fmt::Debug for PermListOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}:", self.principal)?;
        for entry in &self.authorities {
            match entry.marker {
                Some(marker) => writeln!(f, "  {} {marker}", entry.authority)?,
                None => writeln!(f, "  {}", entry.authority)?,
            }
        }
        Ok(())
    }
}

/// List committed permissions plus working-tree pending grants for a principal.
pub fn perm_list(
    repo: &gix::Repository,
    target_ref: &str,
    principal: Option<&str>,
) -> anyhow::Result<PermListOutcome> {
    let config = load_governance_config(repo, target_ref)?;
    let caller = but_authz::resolve_principal_from_env(&config)?;
    let target = principal.unwrap_or_else(|| caller.id().as_str());
    let target_id = PrincipalId::new(target);

    if caller.id() != &target_id {
        let held = but_authz::effective_authority(&caller, &config);
        if !held.contains(but_authz::Authority::AdministrationRead)
            && !held.contains(but_authz::Authority::AdministrationWrite)
        {
            return Err(Denial::missing_permission(
                but_authz::Authority::AdministrationRead,
                &held,
            )
            .into());
        }
    }

    let committed = config
        .principal_authorities(&target_id)
        .cloned()
        .unwrap_or_else(AuthoritySet::empty);
    let working = read_worktree_permissions(repo)?;
    let working_direct = direct_authorities_for_principal(&working, target)?;

    let mut entries = committed
        .iter()
        .map(|authority| PermListEntry {
            authority: authority.name().to_owned(),
            marker: None,
        })
        .collect::<Vec<_>>();

    let committed_names = committed
        .iter()
        .map(Authority::name)
        .collect::<BTreeSet<_>>();
    entries.extend(
        working_direct
            .into_iter()
            .filter(|authority| !committed_names.contains(authority.name()))
            .map(|authority| PermListEntry {
                authority: authority.name().to_owned(),
                marker: Some("PENDING"),
            }),
    );

    Ok(PermListOutcome {
        principal: target.to_owned(),
        authorities: entries,
    })
}

/// Grant direct functional permissions in the working-tree governance config.
pub fn perm_grant(
    repo: &gix::Repository,
    target_ref: &str,
    principal: &str,
    authorities: &[&str],
) -> anyhow::Result<PermWriteOutcome> {
    let parsed = parse_authorities(authorities)?;
    enforce_administration_write_gate(repo, target_ref)?;

    let mut permissions = load_permissions_for_write(repo, target_ref)?;
    let principal_wire = principal_entry_mut(&mut permissions, principal)?;
    let mut changed = false;
    for authority in &parsed {
        let token = authority.name();
        if !principal_wire
            .permissions
            .iter()
            .any(|existing| existing == token)
        {
            principal_wire.permissions.push(token.to_owned());
            changed = true;
        }
    }

    if changed {
        write_worktree_permissions(repo, &permissions)?;
    }

    Ok(write_outcome(principal, &parsed))
}

/// Revoke direct functional permissions from the working-tree governance config.
pub fn perm_revoke(
    repo: &gix::Repository,
    target_ref: &str,
    principal: &str,
    authorities: &[&str],
) -> anyhow::Result<PermWriteOutcome> {
    let parsed = parse_authorities(authorities)?;
    enforce_administration_write_gate(repo, target_ref)?;

    let mut permissions = load_permissions_for_write(repo, target_ref)?;
    let Some(principal_wire) = permissions
        .principal
        .iter_mut()
        .find(|entry| entry.id == principal)
    else {
        return Ok(write_outcome(principal, &parsed));
    };

    let revoke_tokens = parsed
        .iter()
        .map(|authority| authority.name())
        .collect::<BTreeSet<_>>();
    let before_len = principal_wire.permissions.len();
    principal_wire
        .permissions
        .retain(|token| !revoke_tokens.contains(token.as_str()));

    if principal_wire.permissions.len() != before_len {
        write_worktree_permissions(repo, &permissions)?;
    }

    Ok(write_outcome(principal, &parsed))
}

fn write_outcome(principal: &str, authorities: &[Authority]) -> PermWriteOutcome {
    PermWriteOutcome {
        principal: principal.to_owned(),
        authorities: authorities
            .iter()
            .map(|authority| authority.name().to_owned())
            .collect(),
        caveat: REF_PIN_CAVEAT,
    }
}

fn parse_authorities(authorities: &[&str]) -> anyhow::Result<Vec<Authority>> {
    authorities
        .iter()
        .map(|token| Authority::parse(token))
        .collect::<Result<Vec<_>, _>>()
        .context("parsing permission authority token")
}

fn principal_entry_mut<'a>(
    permissions: &'a mut PermissionsWire,
    principal: &str,
) -> anyhow::Result<&'a mut PrincipalWire> {
    if let Some(position) = permissions
        .principal
        .iter()
        .position(|entry| entry.id == principal)
    {
        return permissions.principal.get_mut(position).ok_or_else(|| {
            anyhow!("principal position disappeared while preparing permissions rewrite")
        });
    }

    permissions.principal.push(PrincipalWire {
        id: principal.to_owned(),
        permissions: Vec::new(),
        role: None,
        groups: Vec::new(),
    });
    permissions
        .principal
        .last_mut()
        .ok_or_else(|| anyhow!("principal entry was not available after seeding"))
}

fn direct_authorities_for_principal(
    permissions: &PermissionsWire,
    principal: &str,
) -> anyhow::Result<Vec<Authority>> {
    let Some(entry) = permissions
        .principal
        .iter()
        .find(|entry| entry.id == principal)
    else {
        return Ok(Vec::new());
    };

    entry
        .permissions
        .iter()
        .map(|token| Authority::parse(token))
        .collect::<Result<Vec<_>, _>>()
        .context("parsing working-tree direct permission token")
}

fn load_permissions_for_write(
    repo: &gix::Repository,
    target_ref: &str,
) -> anyhow::Result<PermissionsWire> {
    let path = worktree_permissions_path(repo)?;
    if path.is_file() {
        return parse_permissions_text(
            &fs::read_to_string(&path)
                .with_context(|| format!("reading working-tree {}", permissions_path()))?,
        );
    }

    parse_permissions_text(&read_committed_permissions(repo, target_ref)?)
}

fn read_worktree_permissions(repo: &gix::Repository) -> anyhow::Result<PermissionsWire> {
    let path = worktree_permissions_path(repo)?;
    if !path.is_file() {
        return Ok(PermissionsWire::default());
    }
    parse_permissions_text(
        &fs::read_to_string(&path)
            .with_context(|| format!("reading working-tree {}", permissions_path()))?,
    )
}

fn parse_permissions_text(text: &str) -> anyhow::Result<PermissionsWire> {
    if text.trim().is_empty() {
        return Ok(PermissionsWire::default());
    }
    toml::from_str::<PermissionsWire>(text)
        .with_context(|| format!("parsing working-tree {}", permissions_path()))
}

fn write_worktree_permissions(
    repo: &gix::Repository,
    permissions: &PermissionsWire,
) -> anyhow::Result<()> {
    let path = worktree_permissions_path(repo)?;
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("{} must have a parent directory", permissions_path()))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("creating working-tree {}", parent.display()))?;
    let encoded = toml::to_string(permissions)
        .with_context(|| format!("serializing working-tree {}", permissions_path()))?;
    fs::write(&path, encoded)
        .with_context(|| format!("writing working-tree {}", permissions_path()))?;
    Ok(())
}

fn worktree_permissions_path(repo: &gix::Repository) -> anyhow::Result<PathBuf> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow!("governance permission writes require a non-bare repository"))?;
    Ok(workdir.join(permissions_path()))
}

fn read_committed_permissions(repo: &gix::Repository, target_ref: &str) -> anyhow::Result<String> {
    let mut reference = repo
        .find_reference(target_ref)
        .with_context(|| format!("resolving target ref {target_ref}"))?;
    let commit = reference
        .peel_to_commit()
        .with_context(|| format!("peeling {target_ref} to a commit"))?;
    let tree = commit
        .tree()
        .with_context(|| format!("reading tree for {target_ref}"))?;
    let Some(entry) = tree
        .lookup_entry_by_path(Path::new(permissions_path()))
        .with_context(|| format!("looking up {} in {target_ref}", permissions_path()))?
    else {
        return Ok(String::new());
    };
    let blob = repo
        .find_blob(entry.id())
        .with_context(|| format!("reading {} blob at {target_ref}", permissions_path()))?;
    let content = std::str::from_utf8(&blob.data)
        .with_context(|| format!("decoding {} at {target_ref} as UTF-8", permissions_path()))?;
    Ok(content.to_owned())
}
