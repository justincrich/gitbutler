use but_authz::{Denial, load_governance_config};
use serde::Serialize;

/// Structured administration-write gate error payload for API callers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AdminWriteGateError {
    /// Stable consumer-facing error code.
    pub code: &'static str,
    /// Human-readable denial message.
    pub message: String,
}

/// Enforce authorization before mutating governed configuration.
///
/// The guard reads governance files from the committed `target_ref`, resolves
/// the acting principal from `BUT_AGENT_HANDLE`, and requires
/// `administration:write`.
pub fn enforce_administration_write_gate(
    repo: &gix::Repository,
    target_ref: &str,
) -> anyhow::Result<()> {
    let cfg = load_governance_config(repo, target_ref)?;
    let principal = but_authz::resolve_principal_from_env(&cfg)?;

    but_authz::authorize(&principal, but_authz::Authority::AdministrationWrite, &cfg)?;

    Ok(())
}

/// Extract a structured administration-write gate payload from an error chain.
pub fn classify_error(err: &anyhow::Error) -> Option<AdminWriteGateError> {
    if let Some(denial) = err.downcast_ref::<Denial>() {
        return Some(AdminWriteGateError {
            code: denial.code,
            message: denial.message.clone(),
        });
    }

    err.downcast_ref::<but_authz::ConfigError>()
        .map(|error| AdminWriteGateError {
            code: error.code(),
            message: error.to_string(),
        })
}
