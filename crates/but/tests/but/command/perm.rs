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

#[test]
#[serial_test::serial]
fn perm_denials_include_remediation_hint() -> anyhow::Result<()> {
    let env = perm_env()?;
    let cases = [
        (
            "perm grant",
            env.but("perm grant --principal rust-reviewer reviews:write")
                .env("BUT_AGENT_HANDLE", "rust-implementer")
                .output()?,
            "administration:write",
        ),
        (
            "perm revoke",
            env.but("perm revoke --principal admin administration:write")
                .env("BUT_AGENT_HANDLE", "rust-implementer")
                .output()?,
            "administration:write",
        ),
        (
            "perm list",
            env.but("perm list --principal rust-implementer")
                .env_remove("BUT_AGENT_HANDLE")
                .output()?,
            "BUT_AGENT_HANDLE",
        ),
    ];

    for (verb, output, message_fragment) in cases {
        assert_eq!(
            output.status.code(),
            Some(1),
            "{verb} denial must exit 1, got status {:?}",
            output.status
        );
        let envelope = parse_cli_error_envelope(&output, verb);
        assert_eq!(
            envelope.code, "perm.denied",
            "{verb} denial must include the stable perm.denied code"
        );
        assert!(
            envelope.message.contains(message_fragment),
            "{verb} denial message must contain {message_fragment:?}, got: {}",
            envelope.message
        );
        assert!(
            !envelope.remediation_hint.trim().is_empty(),
            "{verb} denial must include a non-empty remediation_hint"
        );
        println!(
            "seeded {verb} CLI denial: code={}, message={}, remediation_hint={}",
            envelope.code, envelope.message, envelope.remediation_hint
        );
    }

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

[[principal]]
id = "rust-reviewer"
permissions = ["contents:read"]
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

#[derive(Debug, Clone)]
struct CliErrorEnvelope {
    code: String,
    message: String,
    remediation_hint: String,
}

fn parse_cli_error_envelope(output: &std::process::Output, reason: &str) -> CliErrorEnvelope {
    parse_cli_error_envelope_opt(output).unwrap_or_else(|| {
        panic!(
            "{reason}; stderr must contain a parseable CLI JSON error envelope, got: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn parse_cli_error_envelope_opt(output: &std::process::Output) -> Option<CliErrorEnvelope> {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let json = stderr.lines().find_map(json_object_from_line)?;
    let value = serde_json::from_str::<serde_json::Value>(json).ok()?;
    let error = value.get("error")?;
    let code = error.get("code")?.as_str()?.to_owned();
    let message = error.get("message")?.as_str()?.to_owned();
    let remediation_hint = error.get("remediation_hint")?.as_str()?.to_owned();
    Some(CliErrorEnvelope {
        code,
        message,
        remediation_hint,
    })
}

fn json_object_from_line(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    let start = trimmed.find('{')?;
    let end = trimmed.rfind('}')?;
    (start <= end).then_some(&trimmed[start..=end])
}
