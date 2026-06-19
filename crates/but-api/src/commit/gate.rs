use std::path::PathBuf;

use anyhow::bail;
use bstr::ByteSlice as _;
use but_authz::{Authority, Denial, Principal, authorize, load_governance_config};
use but_rebase::graph_rebase::mutate::RelativeTo;
use gix::refs::FullName;
use serde::Serialize;

/// Structured commit-gate error payload for CLI and API callers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommitGateError {
    /// Stable consumer-facing error code.
    pub code: &'static str,
    /// Human-readable denial message.
    pub message: String,
}

/// Ref-backed authorization target for commit creation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitGateTarget {
    config_ref: FullName,
    protected_branch: Option<String>,
}

impl CommitGateTarget {
    /// Gate a direct commit to a branch ref, including branch protection.
    pub fn direct_ref(config_ref: FullName) -> Self {
        Self {
            protected_branch: Some(config_ref.shorten().to_string()),
            config_ref,
        }
    }

    /// Gate a commit operation using committed config from `config_ref`, with branch protection N/A.
    pub fn config_only(config_ref: FullName) -> Self {
        Self {
            config_ref,
            protected_branch: None,
        }
    }
}

/// Enforce commit authorization for a ref-aware commit target.
///
/// If the target ref carries committed governance files, the gate reads that
/// governance configuration, resolves the acting principal from
/// `BUT_AGENT_HANDLE`, requires `contents:write`, and rejects direct commits to
/// protected branches before callers take write guards or mutate repository
/// state. Target refs with no committed governance files remain non-governed.
pub fn enforce_commit_gate(repo: &gix::Repository, relative_to: &RelativeTo) -> anyhow::Result<()> {
    let target = match target_ref(relative_to)? {
        Some(target) => CommitGateTarget::direct_ref(target),
        None => bail!("target ref is required to load governance config for commit authorization"),
    };
    enforce_commit_gate_for_target(repo, &target)
}

/// Enforce commit authorization for a resolved commit target.
pub fn enforce_commit_gate_for_target(
    repo: &gix::Repository,
    target: &CommitGateTarget,
) -> anyhow::Result<()> {
    let full_name = target.config_ref.as_bstr().to_str()?;
    if !has_governance_marker(repo, full_name)? {
        return Ok(());
    }

    let cfg = load_governance_config(repo, full_name)?;
    let principal = but_authz::resolve_principal_from_env(&cfg)?;

    authorize(&principal, Authority::ContentsWrite, &cfg)?;

    if let Some(branch_name) = &target.protected_branch
        && cfg
            .branch(branch_name)
            .is_some_and(|branch| branch.protected())
    {
        return Err(branch_protected(&principal, branch_name).into());
    }

    Ok(())
}

/// Extract a structured gate payload from an error chain.
pub fn classify_error(err: &anyhow::Error) -> Option<CommitGateError> {
    if let Some(denial) = err.downcast_ref::<Denial>() {
        return Some(CommitGateError {
            code: denial.code,
            message: denial.message.clone(),
        });
    }

    err.downcast_ref::<but_authz::ConfigError>()
        .map(|error| CommitGateError {
            code: error.code(),
            message: error.to_string(),
        })
}

fn target_ref(relative_to: &RelativeTo) -> anyhow::Result<Option<FullName>> {
    let RelativeTo::Reference(name) = relative_to else {
        return Ok(None);
    };
    name.as_bstr().to_str()?;
    Ok(Some(name.clone()))
}

fn has_governance_marker(repo: &gix::Repository, target_ref: &str) -> anyhow::Result<bool> {
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

    Ok(
        has_tree_entry(&tree, governance_path("permissions").as_path())?
            || has_tree_entry(&tree, governance_path("gates").as_path())?,
    )
}

fn has_tree_entry(tree: &gix::Tree<'_>, path: &std::path::Path) -> anyhow::Result<bool> {
    Ok(tree.lookup_entry_by_path(path)?.is_some())
}

fn governance_path(stem: &str) -> PathBuf {
    let directory = [".git", "butler"].concat();
    let file_name = [stem, ".", "toml"].concat();
    PathBuf::from(directory).join(file_name)
}

fn branch_protected(principal: &Principal, branch_name: &str) -> Denial {
    Denial {
        code: "branch.protected",
        message: format!(
            "direct commits to protected branch \"{branch_name}\" are denied for principal \"{}\"; land changes through a reviewed merge",
            principal.id().as_str()
        ),
        remediation_hint: format!(
            "open a reviewed merge into {branch_name} instead of committing directly"
        ),
    }
}
