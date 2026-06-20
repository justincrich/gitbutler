use crate::utils::Sandbox;

const REF_PIN_CAVEAT: &str = "takes effect once committed to the target branch";

#[test]
#[serial_test::serial]
fn perm_cli_grant_revoke_denial_and_as_rejection() -> anyhow::Result<()> {
    let env = perm_env()?;
    let repo = env.open_repo()?;
    let main_before = ref_id(&repo, "refs/heads/main")?;

    let grant = env
        .but("perm grant --principal rust-implementer reviews:write")
        .env("BUT_AGENT_HANDLE", "admin")
        .output()?;
    assert!(grant.status.success(), "admin perm grant must succeed");
    let grant_stdout = String::from_utf8_lossy(&grant.stdout);
    assert!(
        grant_stdout.contains(REF_PIN_CAVEAT),
        "grant stdout must include the ref-pin caveat, got: {grant_stdout}"
    );
    assert_eq!(
        ref_id(&repo, "refs/heads/main")?,
        main_before,
        "CLI perm grant must not commit or move refs/heads/main"
    );
    let worktree_permissions = std::fs::read_to_string(
        env.projects_root()
            .join(".gitbutler")
            .join("permissions.toml"),
    )?;
    assert!(
        worktree_permissions.contains("reviews:write"),
        "CLI grant must write the working-tree permissions.toml"
    );

    let list = env
        .but("perm list --principal rust-implementer")
        .env("BUT_AGENT_HANDLE", "admin")
        .output()?;
    assert!(list.status.success(), "admin perm list must succeed");
    let list_stdout = String::from_utf8_lossy(&list.stdout);
    assert!(
        list_stdout.contains("contents:write") && list_stdout.contains("reviews:write"),
        "list stdout must include committed and pending authorities, got: {list_stdout}"
    );
    assert!(
        list_stdout.contains("PENDING"),
        "list stdout must mark uncommitted reviews:write as PENDING, got: {list_stdout}"
    );

    let revoke = env
        .but("perm revoke --principal rust-implementer contents:write")
        .env("BUT_AGENT_HANDLE", "admin")
        .output()?;
    assert!(revoke.status.success(), "admin perm revoke must succeed");
    let revoke_stdout = String::from_utf8_lossy(&revoke.stdout);
    assert!(
        revoke_stdout.contains(REF_PIN_CAVEAT),
        "revoke stdout must include the ref-pin caveat, got: {revoke_stdout}"
    );

    let denied = env
        .but("perm grant --principal rust-implementer administration:write")
        .env("BUT_AGENT_HANDLE", "rust-implementer")
        .output()?;
    assert!(!denied.status.success(), "non-admin grant must fail");
    let denied_stderr = String::from_utf8_lossy(&denied.stderr);
    assert!(
        denied_stderr.contains("perm.denied") && denied_stderr.contains("administration:write"),
        "non-admin denial must be structured and name administration:write, got: {denied_stderr}"
    );

    let override_attempt = env
        .but("perm grant --as admin --principal rust-implementer reviews:write")
        .env("BUT_AGENT_HANDLE", "rust-implementer")
        .output()?;
    assert!(
        !override_attempt.status.success(),
        "`--as` must be rejected before any governed action"
    );
    let override_stderr = String::from_utf8_lossy(&override_attempt.stderr);
    assert!(
        override_stderr.contains("--as"),
        "clap rejection must name the unsupported --as argument, got: {override_stderr}"
    );
    assert!(
        !override_stderr.contains("perm.denied"),
        "--as rejection must happen before governance authorization, got: {override_stderr}"
    );

    Ok(())
}

fn perm_env() -> anyhow::Result<Sandbox> {
    let env = Sandbox::open_scenario_with_target_and_default_settings("one-stack")?;
    env.invoke_bash(
        r#"
git branch -f main origin/main
git checkout main
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "admin"
permissions = ["administration:write", "merge"]

[[principal]]
id = "rust-implementer"
permissions = ["contents:write"]
EOF
cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF
git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "governance config"
"#,
    );
    env.but("setup").assert().success();
    Ok(env)
}

fn ref_id(repo: &gix::Repository, ref_name: &str) -> anyhow::Result<gix::ObjectId> {
    let mut reference = repo.find_reference(ref_name)?;
    Ok(reference.peel_to_commit()?.id)
}
