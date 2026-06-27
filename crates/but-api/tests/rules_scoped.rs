use anyhow::Context as _;
use but_core::ref_metadata::StackId;
use but_rules::{
    Action, CreateRuleRequest, Filter, ImplicitOperation, Operation, StackTarget, Trigger,
    WorkspaceRule,
};
use serde_json::{Value, json};

const TARGET_REF: &str = "refs/remotes/origin/main";

#[test]
fn list_workspace_rules_scoped_none_equals_existing() -> anyhow::Result<()> {
    let (repo, _tmp) = but_testsupport::writable_scenario("checkout-head-info");
    let mut ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();
    let seeded = seed_principal_scoped_rules(&mut ctx)?;

    let baseline = but_api::legacy::rules::list_workspace_rules(&ctx, None)?;
    let scoped = but_api::legacy::rules::list_workspace_rules_scoped(&ctx, None)?;

    assert_eq!(
        rule_ids(&scoped),
        rule_ids(&baseline),
        "principalId=None must return the same rule ids, in the same order, as the existing workspace rules query"
    );
    assert_eq!(
        rule_ids(&scoped),
        vec![
            seeded.rule_a.id(),
            seeded.rule_b.id(),
            seeded.rule_global.id()
        ],
        "the seeded agent-A, agent-B, and workspace-global rules must all remain visible in the unscoped view"
    );
    assert_eq!(
        session_ids(&scoped),
        session_ids(&baseline),
        "principalId=None must preserve the existing workspace rule entity shape, including session associations"
    );

    Ok(())
}

#[test]
fn list_workspace_rules_scoped_preserves_principal_path_labels() -> anyhow::Result<()> {
    let (repo, _tmp) = but_testsupport::writable_scenario("checkout-head-info");
    let mut ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();
    let out_of_workspace = StackId::generate();
    let rule = create_rule_for_path(
        &mut ctx,
        "agent-A",
        "capstone-agent-a-only",
        Action::Explicit(Operation::Assign {
            target: StackTarget::StackId(out_of_workspace.to_string()),
        }),
    )?;

    let unscoped = but_api::legacy::rules::list_workspace_rules(&ctx, None)?;
    assert!(
        !rule_ids(&unscoped).contains(&rule.id()),
        "unscoped workspace rules must keep hiding session rules assigned to stacks outside the workspace"
    );

    let scoped = but_api::legacy::rules::list_workspace_rules_scoped(&ctx, Some("agent-A"))?;
    assert_eq!(
        rule_ids(&scoped),
        vec![rule.id()],
        "principal-scoped rules must not lose the selected principal's rule before the UI can render its labels"
    );
    assert_eq!(
        path_labels(&scoped)?,
        vec!["capstone-agent-a-only".to_owned()],
        "principal-scoped rules must preserve the pathMatchesRegex label consumed by RulesList"
    );

    Ok(())
}

#[test]
fn list_workspace_rules_scoped_some_narrows_to_principal() -> anyhow::Result<()> {
    let (repo, _tmp) = but_testsupport::writable_scenario("checkout-head-info");
    let mut ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();
    let seeded = seed_principal_scoped_rules(&mut ctx)?;

    let agent_a = but_api::legacy::rules::list_workspace_rules_scoped(&ctx, Some("agent-A"))?;
    assert_eq!(
        rule_ids(&agent_a),
        vec![seeded.rule_a.id()],
        "agent-A scope must include only the rule whose Claude Code session association is agent-A"
    );
    assert_eq!(
        session_ids(&agent_a),
        vec![Some("agent-A".to_owned())],
        "scoped results must keep the same WorkspaceRule entity shape and session association"
    );

    let agent_b = but_api::legacy::rules::list_workspace_rules_scoped(&ctx, Some("agent-B"))?;
    assert_eq!(
        rule_ids(&agent_b),
        vec![seeded.rule_b.id()],
        "agent-B scope must include only the rule whose Claude Code session association is agent-B"
    );

    Ok(())
}

#[test]
#[serial_test::serial]
fn rules_scoped_cmd_uses_principal_id_and_returns_path_label() -> anyhow::Result<()> {
    let (repo, _tmp) = rules_governance_repo();
    context_with_target(&repo)?;
    let project_id = project_id_for(&repo)?;

    create_rule_cmd(
        &project_id,
        "agent-A",
        "capstone-agent-a-only",
        Action::Implicit(ImplicitOperation::AssignToAppropriateBranch),
    )?;
    create_rule_cmd(
        &project_id,
        "agent-B",
        "capstone-agent-b-only",
        Action::Implicit(ImplicitOperation::AssignToAppropriateBranch),
    )?;

    let response = temp_env::with_var("BUT_AGENT_HANDLE", Some("admin-reader"), || {
        but_api::legacy::rules::list_workspace_rules_cmd(json!({
            "projectId": project_id,
            "principalId": "agent-A",
        }))
    })?;

    assert_eq!(
        json_path_labels(&response),
        vec!["capstone-agent-a-only".to_owned()],
        "the server-facing command must deserialize principalId and return only the selected principal's path label"
    );
    assert!(
        !json_path_labels(&response).contains(&"capstone-agent-b-only".to_owned()),
        "selecting agent-A must not leak agent-B's scoped rule into the command response"
    );

    Ok(())
}

#[test]
fn list_workspace_rules_scoped_unknown_principal_empty() -> anyhow::Result<()> {
    let (repo, _tmp) = but_testsupport::writable_scenario("checkout-head-info");
    let mut ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();
    seed_principal_scoped_rules(&mut ctx)?;

    let scoped = but_api::legacy::rules::list_workspace_rules_scoped(&ctx, Some("agent-Z"))?;

    assert!(
        scoped.is_empty(),
        "a principal with no associated rules must return an empty list, not all workspace rules"
    );

    Ok(())
}

#[test]
#[serial_test::serial]
fn list_workspace_rules_scoped_cross_principal_not_disclosed() -> anyhow::Result<()> {
    let (repo, _tmp) = rules_governance_repo();
    let mut ctx = context_with_target(&repo)?;
    let seeded = seed_principal_scoped_rules(&mut ctx)?;

    let self_scoped = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("agent-A")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || but_api::legacy::rules::list_workspace_rules_scoped_for_caller(&ctx, Some("agent-A")),
    )?;
    assert_eq!(
        rule_ids(&self_scoped),
        vec![seeded.rule_a.id()],
        "a non-admin caller must be allowed to scope the rules query to its own principal"
    );

    let cross_scoped = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("agent-A")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || but_api::legacy::rules::list_workspace_rules_scoped_for_caller(&ctx, Some("agent-B")),
    )?;
    assert!(
        cross_scoped.is_empty(),
        "a non-admin caller requesting another principal must receive an empty list, not the other principal's rules"
    );

    let admin_scoped = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("admin-reader")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || but_api::legacy::rules::list_workspace_rules_scoped_for_caller(&ctx, Some("agent-B")),
    )?;
    assert_eq!(
        rule_ids(&admin_scoped),
        vec![seeded.rule_b.id()],
        "an administration:read caller must be allowed to scope the rules query to another principal"
    );

    Ok(())
}

struct SeededRules {
    rule_a: WorkspaceRule,
    rule_b: WorkspaceRule,
    rule_global: WorkspaceRule,
}

fn seed_principal_scoped_rules(ctx: &mut but_ctx::Context) -> anyhow::Result<SeededRules> {
    let rule_a = create_rule(ctx, Some("agent-A"))?;
    let rule_b = create_rule(ctx, Some("agent-B"))?;
    let rule_global = create_rule(ctx, None)?;

    Ok(SeededRules {
        rule_a,
        rule_b,
        rule_global,
    })
}

fn create_rule(
    ctx: &mut but_ctx::Context,
    session_id: Option<&str>,
) -> anyhow::Result<WorkspaceRule> {
    let mut filters = Vec::new();
    if let Some(session_id) = session_id {
        filters.push(Filter::ClaudeCodeSessionId(session_id.to_owned()));
    }

    but_api::legacy::rules::create_workspace_rule(
        ctx,
        CreateRuleRequest {
            trigger: Trigger::ClaudeCodeHook,
            filters,
            action: Action::Implicit(ImplicitOperation::AssignToAppropriateBranch),
        },
    )
}

fn create_rule_for_path(
    ctx: &mut but_ctx::Context,
    session_id: &str,
    path: &str,
    action: Action,
) -> anyhow::Result<WorkspaceRule> {
    but_api::legacy::rules::create_workspace_rule(
        ctx,
        CreateRuleRequest {
            trigger: Trigger::ClaudeCodeHook,
            filters: vec![
                Filter::ClaudeCodeSessionId(session_id.to_owned()),
                path_filter(path)?,
            ],
            action,
        },
    )
}

fn create_rule_cmd(
    project_id: &but_ctx::ProjectHandleOrLegacyProjectId,
    session_id: &str,
    path: &str,
    action: Action,
) -> anyhow::Result<Value> {
    but_api::legacy::rules::create_workspace_rule_cmd(json!({
        "projectId": project_id,
        "request": CreateRuleRequest {
            trigger: Trigger::ClaudeCodeHook,
            filters: vec![
                Filter::ClaudeCodeSessionId(session_id.to_owned()),
                path_filter(path)?,
            ],
            action,
        },
    }))
}

fn path_filter(path: &str) -> anyhow::Result<Filter> {
    Ok(serde_json::from_value(json!({
        "type": "pathMatchesRegex",
        "subject": path,
    }))?)
}

fn rule_ids(rules: &[WorkspaceRule]) -> Vec<String> {
    rules.iter().map(WorkspaceRule::id).collect()
}

fn session_ids(rules: &[WorkspaceRule]) -> Vec<Option<String>> {
    rules.iter().map(WorkspaceRule::session_id).collect()
}

fn path_labels(rules: &[WorkspaceRule]) -> anyhow::Result<Vec<String>> {
    Ok(json_path_labels(&serde_json::to_value(rules)?))
}

fn json_path_labels(rules: &Value) -> Vec<String> {
    rules
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|rule| {
            rule.get("filters")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter(|filter| filter.get("type").and_then(Value::as_str) == Some("pathMatchesRegex"))
        .filter_map(|filter| filter.get("subject").and_then(Value::as_str))
        .map(ToOwned::to_owned)
        .collect()
}

fn rules_governance_repo() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "admin-reader"
permissions = ["administration:read"]

[[principal]]
id = "agent-A"
permissions = ["contents:read"]

[[principal]]
id = "agent-B"
permissions = ["contents:read"]
EOF

cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF

git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "rules governance config"
git update-ref refs/remotes/origin/main refs/heads/main
"#,
        &repo,
    );
    (repo, tmp)
}

fn context_with_target(repo: &gix::Repository) -> anyhow::Result<but_ctx::Context> {
    let ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
    let mut project_meta = ctx.project_meta()?;
    project_meta.target_ref = Some(TARGET_REF.try_into()?);
    project_meta.target_commit_id = Some(ref_id(repo, TARGET_REF)?);
    ctx.set_project_meta(project_meta)?;
    Ok(ctx)
}

fn ref_id(repo: &gix::Repository, ref_name: &str) -> anyhow::Result<gix::ObjectId> {
    let mut reference = repo
        .find_reference(ref_name)
        .with_context(|| format!("resolving target ref {ref_name}"))?;
    Ok(reference
        .peel_to_commit()
        .with_context(|| format!("peeling {ref_name} to a commit"))?
        .id)
}

fn project_id_for(
    repo: &gix::Repository,
) -> anyhow::Result<but_ctx::ProjectHandleOrLegacyProjectId> {
    let handle = but_ctx::ProjectHandle::from_path(repo.git_dir())?;
    Ok(but_ctx::ProjectHandleOrLegacyProjectId::ProjectHandle(
        handle,
    ))
}
