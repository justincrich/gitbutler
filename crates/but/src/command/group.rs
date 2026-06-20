//! Command implementation for governed group management.

use anyhow::Context as _;
use but_api::legacy::governance::{GroupListOutcome, GroupWriteOutcome};
use but_ctx::Context;

use crate::{CliError, args::group::Subcommands, utils::OutputChannel};

/// Execute `but group`.
pub async fn exec(
    ctx: &mut Context,
    out: &mut OutputChannel,
    cmd: Option<Subcommands>,
) -> Result<(), CliError> {
    let target_ref = resolve_target_ref(ctx).map_err(CliError::from)?;
    let repo = ctx.repo.get().map_err(CliError::from)?;

    match cmd.unwrap_or(Subcommands::List) {
        Subcommands::List => {
            let list = but_api::legacy::governance::group_list(&repo, &target_ref)
                .map_err(governance_cli_error)?;
            write_list(out, &list).map_err(CliError::from)
        }
        Subcommands::Create { name, authorities } => {
            let authority_refs = authorities.iter().map(String::as_str).collect::<Vec<_>>();
            let create = but_api::legacy::governance::group_create(
                &repo,
                &target_ref,
                &name,
                &authority_refs,
            )
            .map_err(governance_cli_error)?;
            write_mutation(out, "created", &create).map_err(CliError::from)
        }
        Subcommands::Grant { name, authorities } => {
            let authority_refs = authorities.iter().map(String::as_str).collect::<Vec<_>>();
            let grant = but_api::legacy::governance::group_grant(
                &repo,
                &target_ref,
                &name,
                &authority_refs,
            )
            .map_err(governance_cli_error)?;
            write_mutation(out, "granted", &grant).map_err(CliError::from)
        }
        Subcommands::AddMember { name, member } => {
            let add =
                but_api::legacy::governance::group_add_member(&repo, &target_ref, &name, &member)
                    .map_err(governance_cli_error)?;
            write_mutation(out, "added member", &add).map_err(CliError::from)
        }
        Subcommands::RemoveMember { name, member } => {
            let remove = but_api::legacy::governance::group_remove_member(
                &repo,
                &target_ref,
                &name,
                &member,
            )
            .map_err(governance_cli_error)?;
            write_mutation(out, "removed member", &remove).map_err(CliError::from)
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
    outcome: &GroupWriteOutcome,
) -> anyhow::Result<()> {
    if let Some(out) = out.for_human_or_shell() {
        let detail = if outcome.authorities.is_empty() {
            outcome.member.as_deref().map_or_else(
                || outcome.group.clone(),
                |member| format!("{member} in {}", outcome.group),
            )
        } else {
            format!("{} for {}", outcome.authorities.join(", "), outcome.group)
        };
        writeln!(out, "{verb} {detail}; {}.", outcome.caveat)?;
    } else if let Some(out) = out.for_json() {
        out.write_value(outcome)?;
    }
    Ok(())
}

fn write_list(out: &mut OutputChannel, list: &GroupListOutcome) -> anyhow::Result<()> {
    if let Some(out) = out.for_human_or_shell() {
        for group in &list.groups {
            writeln!(out, "{}:", group.name)?;
            for authority in &group.authorities {
                writeln!(out, "  grant {authority}")?;
            }
            for member in &group.members {
                writeln!(out, "  member {member}")?;
            }
        }
    } else if let Some(out) = out.for_json() {
        out.write_value(list)?;
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
