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
            let list = but_api::legacy::governance::perm_list_with_repo(
                &repo,
                &target_ref,
                principal.as_deref(),
            )
            .map_err(governance_cli_error)?;
            write_list(out, &list).map_err(CliError::from)
        }
        Subcommands::Grant {
            principal,
            authorities,
        } => {
            let authority_refs = authorities.iter().map(String::as_str).collect::<Vec<_>>();
            let grant = but_api::legacy::governance::perm_grant_with_repo(
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
            let revoke = but_api::legacy::governance::perm_revoke_with_repo(
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
    // STEER-005: admin-write actor_correctable denials carry the full
    // steering payload (class/held_permissions/authorized_actions/do_not)
    // plus remediation_hint, rendered through steer_envelope_from_parts().
    // The AdminWriteGateError carrier (from config_mutate::classify_error)
    // supplies the four steering fields; remediation_hint is sourced from
    // the underlying Denial. Best-effort: a serialization fault still emits
    // code/message/remediation_hint + exit 1 (invariant §9.5).
    if let Some(gate_error) = but_api::legacy::config_mutate::classify_error(&err) {
        let remediation_hint = err
            .downcast_ref::<but_authz::Denial>()
            .map(|denial| denial.remediation_hint.as_str());
        let envelope = but_authz::steer_envelope_from_parts(
            gate_error.code,
            &gate_error.message,
            remediation_hint,
            gate_error.class,
            &gate_error.held_permissions,
            &gate_error.authorized_actions,
            gate_error.do_not,
        );
        return anyhow::anyhow!("{}", serde_json::json!({ "error": envelope })).into();
    }

    // ConfigInvalid-sourced (operator_required config.invalid): keep the
    // legacy code/message/remediation_hint shape via classify_governance_error.
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
