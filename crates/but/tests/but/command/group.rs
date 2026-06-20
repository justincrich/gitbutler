use crate::utils::Sandbox;

const REF_PIN_CAVEAT: &str = "takes effect once committed to the target branch";

#[test]
fn group_no_delete_surface_in_sprint_05() -> anyhow::Result<()> {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let group_args = std::fs::read_to_string(manifest_dir.join("src/args/group.rs"))?;
    let group_command = std::fs::read_to_string(manifest_dir.join("src/command/group.rs"))?;
    let governance = std::fs::read_to_string(
        manifest_dir
            .parent()
            .and_then(std::path::Path::parent)
            .ok_or_else(|| anyhow::anyhow!("crate manifest must live below the workspace root"))?
            .join("crates/but-api/src/legacy/governance.rs"),
    )?;

    assert!(
        !group_args.contains("Delete"),
        "Sprint 05 must not expose a group Delete clap variant"
    );
    assert!(
        !governance.contains("group_delete"),
        "Sprint 05 must not expose a group_delete API function"
    );
    for (path, source) in [
        ("crates/but/src/args/group.rs", group_args.as_str()),
        ("crates/but/src/command/group.rs", group_command.as_str()),
        (
            "crates/but-api/src/legacy/governance.rs",
            governance.as_str(),
        ),
    ] {
        assert!(
            !source.contains("todo!()") && !source.contains("unimplemented!()"),
            "{path} must not contain a group deletion placeholder"
        );
    }

    println!("Sprint 05 group surface has no delete variant, API, or placeholder");
    Ok(())
}

#[test]
#[serial_test::serial]
fn group_cli_create_grant_members_list_denial_and_no_delete() -> anyhow::Result<()> {
    let env = group_env()?;
    let repo = env.open_repo()?;
    let main_before = ref_id(&repo, "refs/heads/main")?;

    let create = env
        .but("group create release-captains --permissions reviews:write")
        .env("BUT_AGENT_HANDLE", "admin")
        .output()?;
    assert!(create.status.success(), "admin group create must succeed");
    let create_stdout = String::from_utf8_lossy(&create.stdout);
    assert!(
        create_stdout.contains("reviews:write") && create_stdout.contains(REF_PIN_CAVEAT),
        "create stdout must include the ref-pin caveat, got: {create_stdout}"
    );

    let grant = env
        .but("group grant release-captains administration:write")
        .env("BUT_AGENT_HANDLE", "admin")
        .output()?;
    assert!(grant.status.success(), "admin group grant must succeed");
    let grant_stdout = String::from_utf8_lossy(&grant.stdout);
    assert!(
        grant_stdout.contains(REF_PIN_CAVEAT),
        "grant stdout must include the ref-pin caveat, got: {grant_stdout}"
    );

    let add_member = env
        .but("group add-member release-captains rust-implementer")
        .env("BUT_AGENT_HANDLE", "admin")
        .output()?;
    assert!(
        add_member.status.success(),
        "admin group add-member must succeed"
    );
    let add_member_stdout = String::from_utf8_lossy(&add_member.stdout);
    assert!(
        add_member_stdout.contains(REF_PIN_CAVEAT),
        "add-member stdout must include the ref-pin caveat, got: {add_member_stdout}"
    );

    assert_eq!(
        ref_id(&repo, "refs/heads/main")?,
        main_before,
        "CLI group mutations must not commit or move refs/heads/main"
    );

    let list = env
        .but("group list")
        .env("BUT_AGENT_HANDLE", "admin-reader")
        .output()?;
    assert!(list.status.success(), "admin-read group list must succeed");
    let list_stdout = String::from_utf8_lossy(&list.stdout);
    assert!(
        list_stdout.contains("release-captains")
            && list_stdout.contains("administration:write")
            && list_stdout.contains("reviews:write")
            && list_stdout.contains("rust-implementer"),
        "list stdout must include group names, grants, and members, got: {list_stdout}"
    );

    let remove_member = env
        .but("group remove-member release-captains rust-implementer")
        .env("BUT_AGENT_HANDLE", "admin")
        .output()?;
    assert!(
        remove_member.status.success(),
        "admin group remove-member must succeed"
    );
    let after_remove = std::fs::read_to_string(
        env.projects_root()
            .join(".gitbutler")
            .join("permissions.toml"),
    )?;
    assert!(
        group_block(&after_remove, "release-captains")?.contains("administration:write"),
        "remove-member must preserve the group grant"
    );
    assert!(
        !group_block(&after_remove, "release-captains")?.contains("rust-implementer"),
        "remove-member must remove only the named member"
    );

    let denied = env
        .but("group add-member code-reviewers rust-implementer")
        .env("BUT_AGENT_HANDLE", "rust-implementer")
        .output()?;
    assert!(!denied.status.success(), "non-admin add-member must fail");
    let denied_stderr = String::from_utf8_lossy(&denied.stderr);
    assert!(
        denied_stderr.contains("perm.denied") && denied_stderr.contains("administration:write"),
        "non-admin denial must be structured and name administration:write, got: {denied_stderr}"
    );

    let override_attempt = env
        .but("group grant --as admin release-captains reviews:write")
        .env("BUT_AGENT_HANDLE", "rust-implementer")
        .output()?;
    assert!(
        !override_attempt.status.success(),
        "`--as` must be rejected before any governed action"
    );
    let override_stderr = String::from_utf8_lossy(&override_attempt.stderr);
    assert!(
        override_stderr.contains("--as") && !override_stderr.contains("perm.denied"),
        "--as rejection must happen before governance authorization, got: {override_stderr}"
    );

    let delete_attempt = env
        .but("group delete release-captains")
        .env("BUT_AGENT_HANDLE", "admin")
        .output()?;
    assert!(
        !delete_attempt.status.success(),
        "`but group delete` must not be implemented"
    );
    let delete_stderr = String::from_utf8_lossy(&delete_attempt.stderr);
    assert!(
        delete_stderr.contains("delete"),
        "unsupported delete error should name the rejected subcommand, got: {delete_stderr}"
    );

    Ok(())
}

#[test]
#[serial_test::serial]
fn group_denials_include_remediation_hint() -> anyhow::Result<()> {
    let env = group_env()?;
    let cases = [
        (
            "group create",
            env.but("group create new-team --permissions reviews:write")
                .env("BUT_AGENT_HANDLE", "rust-implementer")
                .output()?,
        ),
        (
            "group grant",
            env.but("group grant code-reviewers comments:write")
                .env("BUT_AGENT_HANDLE", "rust-implementer")
                .output()?,
        ),
        (
            "group add-member",
            env.but("group add-member code-reviewers rust-implementer")
                .env("BUT_AGENT_HANDLE", "rust-implementer")
                .output()?,
        ),
        (
            "group remove-member",
            env.but("group remove-member code-reviewers rust-reviewer")
                .env("BUT_AGENT_HANDLE", "rust-implementer")
                .output()?,
        ),
    ];

    for (verb, output) in cases {
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
            envelope.message.contains("administration:write"),
            "{verb} denial message must name administration:write, got: {}",
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

fn group_env() -> anyhow::Result<Sandbox> {
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
id = "admin-reader"
permissions = ["administration:read"]

[[principal]]
id = "rust-implementer"
permissions = ["contents:write"]

[[group]]
name = "code-reviewers"
permissions = ["reviews:write"]
members = ["rust-reviewer"]
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

fn group_block<'a>(toml: &'a str, name: &str) -> anyhow::Result<&'a str> {
    let marker = format!(r#"name = "{name}""#);
    toml.split("[[group]]")
        .skip(1)
        .find(|block| block.contains(&marker))
        .ok_or_else(|| anyhow::anyhow!("expected group block with {marker}"))
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
