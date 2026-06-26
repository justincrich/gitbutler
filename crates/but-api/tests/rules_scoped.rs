use anyhow::Context as _;
use but_rules::{Action, CreateRuleRequest, Filter, ImplicitOperation, Trigger, WorkspaceRule};

const TARGET_REF: &str = "refs/remotes/origin/main";

#[test]
fn list_workspace_rules_scoped_none_equals_existing() -> anyhow::Result<()> {
    let (repo, _tmp) = but_testsupport::writable_scenario("checkout-head-info");
    let mut ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();
    let seeded = seed_principal_scoped_rules(&mut ctx)?;

    let baseline = but_api::legacy::rules::list_workspace_rules(&ctx)?;
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

fn rule_ids(rules: &[WorkspaceRule]) -> Vec<String> {
    rules.iter().map(WorkspaceRule::id).collect()
}

fn session_ids(rules: &[WorkspaceRule]) -> Vec<Option<String>> {
    rules.iter().map(WorkspaceRule::session_id).collect()
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
