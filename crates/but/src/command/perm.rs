//! Command implementation for governed permission management.

use anyhow::Context as _;
use but_api::legacy::governance::{PermListOutcome, PermWriteOutcome};
use but_ctx::Context;

use crate::{CliError, args::perm::Subcommands, utils::OutputChannel};

/// Execute `but perm`.
pub async fn exec(
    ctx: &mut Context,
    out: &mut OutputChannel,
    cmd: Option<Subcommands>,
) -> Result<(), CliError> {
    let target_ref = resolve_target_ref(ctx).map_err(CliError::from)?;
    let repo = ctx.repo.get().map_err(CliError::from)?;

    match cmd.unwrap_or(Subcommands::List { principal: None }) {
        Subcommands::List { principal } => {
            let list =
                but_api::legacy::governance::perm_list(&repo, &target_ref, principal.as_deref())
                    .map_err(governance_cli_error)?;
            write_list(out, &list).map_err(CliError::from)
        }
        Subcommands::Grant {
            principal,
            authorities,
        } => {
            let authority_refs = authorities.iter().map(String::as_str).collect::<Vec<_>>();
            let grant = but_api::legacy::governance::perm_grant(
                &repo,
                &target_ref,
                &principal,
                &authority_refs,
            )
            .map_err(governance_cli_error)?;
            write_mutation(out, "granted", &grant).map_err(CliError::from)
        }
        Subcommands::Revoke {
            principal,
            authorities,
        } => {
            let authority_refs = authorities.iter().map(String::as_str).collect::<Vec<_>>();
            let revoke = but_api::legacy::governance::perm_revoke(
                &repo,
                &target_ref,
                &principal,
                &authority_refs,
            )
            .map_err(governance_cli_error)?;
            write_mutation(out, "revoked", &revoke).map_err(CliError::from)
        }
    }
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

fn write_mutation(
    out: &mut OutputChannel,
    verb: &str,
    outcome: &PermWriteOutcome,
) -> anyhow::Result<()> {
    if let Some(out) = out.for_human_or_shell() {
        writeln!(
            out,
            "{verb} {} for {}; {}.",
            outcome.authorities.join(", "),
            outcome.principal,
            outcome.caveat
        )?;
    } else if let Some(out) = out.for_json() {
        out.write_value(outcome)?;
    }
    Ok(())
}

fn write_list(out: &mut OutputChannel, list: &PermListOutcome) -> anyhow::Result<()> {
    if let Some(out) = out.for_human_or_shell() {
        writeln!(out, "{}:", list.principal)?;
        for entry in &list.authorities {
            match entry.marker {
                Some(marker) => writeln!(out, "  {} {marker}", entry.authority)?,
                None => writeln!(out, "  {}", entry.authority)?,
            }
        }
    } else if let Some(out) = out.for_json() {
        out.write_value(list)?;
    }
    Ok(())
}

fn governance_cli_error(err: anyhow::Error) -> CliError {
    if let Some(gate_error) = but_api::legacy::config_mutate::classify_error(&err) {
        return anyhow::anyhow!(
            "{}",
            serde_json::json!({
                "error": {
                    "code": gate_error.code,
                    "message": gate_error.message,
                }
            })
        )
        .into();
    }

    err.into()
}
