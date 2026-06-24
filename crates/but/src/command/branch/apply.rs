use but_ctx::Context;
use gix::reference::Category;

use crate::utils::OutputChannel;

/// Apply a branch to the workspace, and return the full ref name to it.
pub fn apply(mut ctx: Context, branch_name: &str, out: &mut OutputChannel) -> anyhow::Result<()> {
    {
        let repo = ctx.repo.get()?;
        // Use the SAME governance-aware resolver as the library path
        // (but_api::branch::workspace_config_ref). This closes the G3 asymmetry:
        // a governed repo without target_ref but with committed .gitbutler/*.toml
        // on some branch is still gated from the CLI.
        if let Some(config_ref) = but_api::branch::workspace_config_ref(&ctx, &repo)? {
            but_api::commit::create::gate::enforce_commit_gate_for_target(
                &repo,
                &but_api::commit::create::gate::CommitGateTarget::config_only(config_ref),
            )
            .map_err(commit_gate_cli_error)?;
        }
    }

    let mut guard = ctx.exclusive_worktree_access();
    let reference = {
        let repo = ctx.repo.get()?;
        repo.find_reference(branch_name)?.detach()
    };
    let mut outcome = but_api::branch::apply_with_perm(
        &mut ctx,
        reference.name.as_ref(),
        guard.write_permission(),
    )?;

    if !outcome.conflicting_stacks.is_empty() {
        let short_name = reference.name.shorten();
        let conflicting_stack_names = outcome
            .conflicting_stacks
            .iter()
            .map(|stack| stack.ref_name.shorten().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        anyhow::bail!(
            "'{short_name}' conflicts with existing stack in the workspace: {conflicting_stack_names}"
        );
    }

    if let Some(out) = out.for_human() {
        // Since `applied_branches` is the actual applied branches, turning remotes into local branches,
        // hack it into submission while the legacy version exists that it has to match.
        let special_case_remove_me_once_there_is_no_legacy_apply =
            outcome.applied_branches.len() == 1;
        if special_case_remove_me_once_there_is_no_legacy_apply {
            outcome.applied_branches = vec![reference.name.clone()];
        }
        for name in outcome.applied_branches {
            let short_name = name.shorten();
            let is_remote_reference = name.category().is_some_and(|c| c == Category::RemoteBranch);
            if is_remote_reference {
                writeln!(out, "Applied remote branch '{short_name}' to workspace")
            } else {
                writeln!(out, "Applied branch '{short_name}' to workspace")
            }?;
        }
    } else if let Some(out) = out.for_shell() {
        writeln!(out, "{reference}", reference = reference.name)?;
    }

    if let Some(out) = out.for_json() {
        out.write_value(but_api::json::Reference::from(reference))?;
    }
    Ok(())
}

fn commit_gate_cli_error(err: anyhow::Error) -> anyhow::Error {
    if let Some(gate_error) = but_api::commit::create::gate::classify_error(&err) {
        // STEER-005: render the full steering envelope (class,
        // held_permissions, authorized_actions, do_not) PLUS the
        // long-missing remediation_hint through the shared
        // steer_envelope_from_parts(). The remediation_hint is sourced
        // from the underlying Denial (the carrier drops it during
        // classify_error); best-effort: a serialization fault still emits
        // code/message/remediation_hint + exit 1 (invariant §9.5).
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
        return anyhow::anyhow!("{}", serde_json::json!({ "error": envelope }));
    }

    err
}

#[cfg(test)]
mod tests {
    use crate::{args::OutputFormat, utils::OutputChannel};

    use super::*;

    #[test]
    #[serial_test::serial]
    fn branch_apply_no_target_ungoverned() -> anyhow::Result<()> {
        let env = but_testsupport::Sandbox::open_with_default_settings("one-fork")?;
        env.invoke_bash(
            r#"
git config user.name GitButler
git config user.email gitbutler@example.com
"#,
        );
        let repo = env.open_repo()?;
        let ctx = Context::from_repo(repo.clone())?.with_memory_app_cache();
        assert!(
            ctx.project_meta()?.target_ref.is_none(),
            "no-target fixture must not configure a workspace target ref"
        );
        let mut out = OutputChannel::new(OutputFormat::None);

        temp_env::with_var("BUT_AGENT_HANDLE", Some("ro"), || apply(ctx, "A", &mut out))?;

        Ok(())
    }
}
