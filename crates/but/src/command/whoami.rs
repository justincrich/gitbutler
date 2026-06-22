//! Command implementation for `but whoami` self-scoped discovery.

use anyhow::Context as _;
use but_api::legacy::governance::WhoamiOutcome;
use but_ctx::Context;

use crate::{CliError, utils::OutputChannel};

/// Execute `but whoami`.
pub async fn exec(
    ctx: &mut Context,
    out: &mut OutputChannel,
    principal: Option<String>,
) -> Result<(), CliError> {
    let target_ref = resolve_target_ref(ctx).map_err(CliError::from)?;
    let repo = ctx.repo.get().map_err(CliError::from)?;

    let outcome =
        but_api::legacy::governance::whoami_with_repo(&repo, &target_ref, principal.as_deref())
            .map_err(governance_cli_error)?;
    write_whoami(out, &outcome).map_err(CliError::from)
}

fn resolve_target_ref(ctx: &mut Context) -> anyhow::Result<String> {
    let target = {
        let mut guard = ctx.exclusive_worktree_access();
        but_api::legacy::virtual_branches::get_base_branch_data(ctx, guard.write_permission())?
    };
    let target = target.context("target ref is required to load governance permissions")?;
    let repo = ctx.repo.get()?;
    for candidate in target_ref_candidates(&target) {
        if repo.try_find_reference(&candidate)?.is_some() {
            return Ok(candidate);
        }
    }
    Ok(target.branch_name)
}

fn target_ref_candidates(target: &gitbutler_branch_actions::BaseBranch) -> Vec<String> {
    let mut candidates = Vec::new();
    if !target.short_name.is_empty() {
        candidates.push(format!("refs/heads/{}", target.short_name));
    }
    candidates.push(target.branch_name.clone());
    if !target.branch_name.starts_with("refs/") {
        candidates.push(format!("refs/remotes/{}", target.branch_name));
        candidates.push(format!("refs/heads/{}", target.branch_name));
    }
    candidates
}

fn write_whoami(out: &mut OutputChannel, outcome: &WhoamiOutcome) -> anyhow::Result<()> {
    if let Some(out) = out.for_human_or_shell() {
        writeln!(out, "{}:", outcome.principal)?;
        for authority in &outcome.authorities {
            writeln!(out, "  {authority}")?;
        }
        writeln!(out, "groups:")?;
        for group in &outcome.groups {
            writeln!(out, "  {group}")?;
        }
        writeln!(out, "authorized actions:")?;
        for action in &outcome.authorized_actions {
            writeln!(out, "  {} — {}", action.command, action.effect)?;
        }
    } else if let Some(out) = out.for_json() {
        out.write_value(outcome)?;
    }
    Ok(())
}

fn governance_cli_error(err: anyhow::Error) -> CliError {
    if let Some(gate_error) = but_api::legacy::governance::classify_governance_error(&err) {
        let mut error = serde_json::json!({
            "code": gate_error.code,
            "message": gate_error.message,
        });
        if let Some(remediation_hint) = gate_error.remediation_hint {
            error["remediation_hint"] = remediation_hint.into();
        }
        return anyhow::anyhow!("{}", serde_json::json!({ "error": error })).into();
    }

    err.into()
}
