//! In place of commands.rs
use std::str::FromStr;

use anyhow::Result;
use but_api_macros::but_api;
use but_authz::{Authority, PrincipalId, load_governance_config};
use but_core::ref_metadata::StackId;
use but_ctx::Context;
use but_rules::{
    CreateRuleRequest, UpdateRuleRequest, WorkspaceRule, create_rule, delete_rule, list_rules,
    update_rule,
};
use tracing::instrument;

#[but_api]
#[instrument(err(Debug))]
pub fn create_workspace_rule(
    ctx: &mut Context,
    request: CreateRuleRequest,
) -> Result<WorkspaceRule> {
    let mut guard = ctx.exclusive_worktree_access();
    create_rule(ctx, request, guard.write_permission())
}

#[but_api]
#[instrument(err(Debug))]
pub fn delete_workspace_rule(ctx: &Context, rule_id: String) -> Result<()> {
    let mut db = ctx.db.get_cache_mut()?;
    delete_rule(&mut db, &rule_id)
}

#[but_api]
#[instrument(err(Debug))]
pub fn update_workspace_rule(
    ctx: &mut Context,
    request: UpdateRuleRequest,
) -> Result<WorkspaceRule> {
    let mut guard = ctx.exclusive_worktree_access();
    update_rule(ctx, request, guard.write_permission())
}

#[but_api]
#[instrument(err(Debug))]
pub fn list_workspace_rules(ctx: &Context) -> Result<Vec<WorkspaceRule>> {
    let in_workspace = crate::legacy::workspace::stacks_v3_from_ctx(
        ctx,
        but_workspace::legacy::StacksFilter::InWorkspace,
    )?
    .iter()
    .filter_map(|s| s.id)
    .collect::<Vec<StackId>>();

    // Filter out specifically Codegen related rules that are refering to a stack that is not in the workspace.
    let db = ctx.db.get_cache()?;
    let rules = list_rules(&db)?
        .into_iter()
        .filter(|rule| {
            if let (Some(_), Some(stack_id)) = (
                rule.session_id(),
                rule.target_stack_id()
                    .and_then(|id| StackId::from_str(&id).ok()),
            ) {
                return in_workspace.contains(&stack_id);
            }
            true
        })
        .collect();

    Ok(rules)
}

#[instrument(err(Debug))]
pub fn list_workspace_rules_scoped(
    ctx: &Context,
    principal_id: Option<&str>,
) -> Result<Vec<WorkspaceRule>> {
    let rules = list_workspace_rules(ctx)?;
    let Some(principal_id) = principal_id else {
        return Ok(rules);
    };

    Ok(rules
        .into_iter()
        .filter(|rule| rule.session_id().as_deref() == Some(principal_id))
        .collect())
}

#[instrument(err(Debug))]
pub fn list_workspace_rules_scoped_for_caller(
    ctx: &Context,
    principal_id: Option<&str>,
) -> Result<Vec<WorkspaceRule>> {
    let Some(principal_id) = principal_id else {
        return list_workspace_rules_scoped(ctx, None);
    };

    let repo = ctx.repo.get()?;
    let target_ref = ctx.project_meta()?.target_ref_or_err()?.to_string();
    let config = load_governance_config(&repo, &target_ref)?;
    let caller = but_authz::resolve_principal_from_env(&config)?;
    let target = PrincipalId::new(principal_id);

    if caller.id() != &target {
        let held = but_authz::effective_authority(&caller, &config);
        if !held.contains(Authority::AdministrationRead)
            && !held.contains(Authority::AdministrationWrite)
        {
            return Ok(Vec::new());
        }
    }

    list_workspace_rules_scoped(ctx, Some(principal_id))
}

#[cfg(all(feature = "napi", feature = "legacy"))]
#[napi_derive::napi(ts_return_type = "Promise<Array<any>>", js_name = "listWorkspaceRules")]
pub async fn list_workspace_rules_napi(
    project_id: String,
    principal_id: Option<String>,
) -> napi::Result<serde_json::Value> {
    tokio::task::spawn_blocking(move || {
        let result = (|| -> anyhow::Result<serde_json::Value> {
            let project_id: but_ctx::ProjectHandleOrLegacyProjectId = project_id.parse()?;
            let ctx = Context::try_from(project_id)?.with_memory_app_cache();
            let rules = list_workspace_rules_scoped_for_caller(&ctx, principal_id.as_deref())?;
            Ok(serde_json::to_value(rules)?)
        })();
        result.map_err(napi_error)
    })
    .await
    .map_err(|error| {
        napi::Error::new(
            napi::Status::GenericFailure,
            format!("spawn_blocking join error: {error}"),
        )
    })?
}

#[cfg(all(feature = "napi", feature = "legacy"))]
fn napi_error(error: anyhow::Error) -> napi::Error {
    let context = but_error::AnyhowContextExt::custom_context_or_error_chain(&error);
    let message = context
        .message
        .map(|message| message.to_string())
        .unwrap_or_else(|| format!("{error:#}"));
    napi::Error::new(napi::Status::GenericFailure, message)
}
