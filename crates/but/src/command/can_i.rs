//! Command implementation for `but can-i` authority-hold check.

use anyhow::Context as _;
use but_api::legacy::governance::CanIOutcome;
use but_ctx::Context;

use crate::{CliError, utils::OutputChannel};

/// Execute `but can-i <authority>`.
pub async fn exec(
    ctx: &mut Context,
    out: &mut OutputChannel,
    authority: String,
    principal: Option<String>,
) -> Result<(), CliError> {
    let target_ref = resolve_target_ref(ctx).map_err(CliError::from)?;
    let repo = ctx.repo.get().map_err(CliError::from)?;

    let outcome = but_api::legacy::governance::can_i_with_repo(
        &repo,
        &target_ref,
        &authority,
        principal.as_deref(),
    )
    .map_err(governance_cli_error)?;
    write_can_i(out, &outcome).map_err(CliError::from)
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

fn write_can_i(out: &mut OutputChannel, outcome: &CanIOutcome) -> anyhow::Result<()> {
    if let Some(out) = out.for_human_or_shell() {
        let answer = if outcome.held { "yes" } else { "no" };
        writeln!(
            out,
            "{answer} — {} {} {}",
            outcome.principal,
            if outcome.held { "holds" } else { "lacks" },
            outcome.authority
        )?;
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
