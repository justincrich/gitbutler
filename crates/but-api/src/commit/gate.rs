use anyhow::bail;
use bstr::ByteSlice as _;
use but_authz::{Authority, Denial, Principal, authorize, load_governance_config};
use but_rebase::graph_rebase::mutate::RelativeTo;
use serde::Serialize;

/// Structured commit-gate error payload for CLI and API callers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommitGateError {
    /// Stable consumer-facing error code.
    pub code: &'static str,
    /// Human-readable denial message.
    pub message: String,
}

/// Enforce commit authorization for a ref-aware commit target.
///
/// The gate reads governance configuration from the target ref, resolves the
/// acting principal from `BUT_AGENT_HANDLE`, requires `contents:write`, and
/// rejects direct commits to protected branches before callers take write
/// guards or mutate repository state.
pub fn enforce_commit_gate(repo: &gix::Repository, relative_to: &RelativeTo) -> anyhow::Result<()> {
    let target = match target_ref(relative_to)? {
        Some(target) => target,
        None => bail!("target ref is required to load governance config for commit authorization"),
    };
    let cfg = load_governance_config(repo, target.full_name)?;
    let principal = but_authz::resolve_principal_from_env(&cfg)?;

    authorize(&principal, Authority::ContentsWrite, &cfg)?;

    if cfg
        .branch(&target.branch_name)
        .is_some_and(|branch| branch.protected())
    {
        return Err(branch_protected(&principal, &target.branch_name).into());
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

struct TargetRef<'a> {
    full_name: &'a str,
    branch_name: String,
}

fn target_ref(relative_to: &RelativeTo) -> anyhow::Result<Option<TargetRef<'_>>> {
    let RelativeTo::Reference(name) = relative_to else {
        return Ok(None);
    };
    let full_name = name.as_bstr().to_str()?;
    Ok(Some(TargetRef {
        full_name,
        branch_name: name.shorten().to_string(),
    }))
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
