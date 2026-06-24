use crate::utils::Sandbox;

const REF_PIN_CAVEAT: &str = "takes effect once committed to the target branch";

/// `but group delete` is now a first-class CLI verb backed by the existing
/// admin-gated `group_delete_with_repo` backend. This test replaces the former
/// `group_no_delete_cli_verb_surface` guard (which asserted the verb was
/// intentionally absent) and proves the positive behavior:
///
/// - non-admin delete is denied with the stable `perm.denied` code (authz runs
///   before the group lookup, so the denial holds even for an existing group),
/// - admin delete exits 0 and prints the ref-pin caveat (consistent with the
///   other group write verbs),
/// - the deleted group no longer appears in `but group list`.
///
/// The group is created via the CLI first so it lands in the working-tree
/// `permissions.toml` that `group list` reads (committed-only groups are not
/// materialized into the working tree by `but setup`).
#[test]
#[serial_test::serial]
fn group_cli_delete_admin_succeeds_and_non_admin_denied() -> anyhow::Result<()> {
    let env = group_env()?;

    // Seed a group via the CLI so it is present in the working-tree config.
    let create = env
        .but("group create delete-target --permissions reviews:write")
        .env("BUT_AGENT_HANDLE", "admin")
        .output()?;
    assert!(create.status.success(), "admin group create must succeed");
    let list_before = env
        .but("group list")
        .env("BUT_AGENT_HANDLE", "admin-reader")
        .output()?;
    assert!(
        list_before.status.success(),
        "admin-read group list must succeed"
    );
    let list_before_stdout = String::from_utf8_lossy(&list_before.stdout);
    assert!(
        list_before_stdout.contains("delete-target"),
        "created group must appear in list before delete, got: {list_before_stdout}"
    );

    // Non-admin (contents:write only) must be denied administration:write.
    // The authz gate runs before group lookup, so the group still existing here
    // proves the denial is a real permission denial, not a not-found error.
    let denied = env
        .but("group delete delete-target")
        .env("BUT_AGENT_HANDLE", "rust-implementer")
        .output()?;
    assert_eq!(
        denied.status.code(),
        Some(1),
        "non-admin group delete must exit 1, got status {:?}",
        denied.status
    );
    let envelope = parse_cli_error_envelope(&denied, "non-admin group delete");
    assert_eq!(
        envelope.code, "perm.denied",
        "non-admin delete denial must use the stable perm.denied code"
    );
    assert!(
        envelope.message.contains("administration:write"),
        "non-admin delete denial must name administration:write, got: {}",
        envelope.message
    );
    assert!(
        !envelope.remediation_hint.trim().is_empty(),
        "non-admin delete denial must include a non-empty remediation_hint"
    );

    // The denial must not have removed the group.
    let list_after_denial = env
        .but("group list")
        .env("BUT_AGENT_HANDLE", "admin-reader")
        .output()?;
    let list_after_denial_stdout = String::from_utf8_lossy(&list_after_denial.stdout);
    assert!(
        list_after_denial_stdout.contains("delete-target"),
        "denied delete must leave the group in place, got: {list_after_denial_stdout}"
    );

    // Admin delete must succeed and print the ref-pin caveat, matching the
    // other group write verbs.
    let deleted = env
        .but("group delete delete-target")
        .env("BUT_AGENT_HANDLE", "admin")
        .output()?;
    assert!(
        deleted.status.success(),
        "admin group delete must succeed; got stderr: {}",
        String::from_utf8_lossy(&deleted.stderr)
    );
    let deleted_stdout = String::from_utf8_lossy(&deleted.stdout);
    assert!(
        deleted_stdout.contains("delete-target") && deleted_stdout.contains(REF_PIN_CAVEAT),
        "delete stdout must name the group and include the ref-pin caveat, got: {deleted_stdout}"
    );

    // The deleted group must no longer appear in `but group list`.
    let list_after = env
        .but("group list")
        .env("BUT_AGENT_HANDLE", "admin-reader")
        .output()?;
    assert!(
        list_after.status.success(),
        "admin-read group list must succeed"
    );
    let list_after_stdout = String::from_utf8_lossy(&list_after.stdout);
    assert!(
        !list_after_stdout.contains("delete-target"),
        "deleted group must not appear in list, got: {list_after_stdout}"
    );

    Ok(())
}

#[test]
#[serial_test::serial]
fn group_cli_create_grant_members_list_denial_and_delete() -> anyhow::Result<()> {
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
        delete_attempt.status.success(),
        "admin group delete must succeed; got stderr: {}",
        String::from_utf8_lossy(&delete_attempt.stderr)
    );
    let delete_stdout = String::from_utf8_lossy(&delete_attempt.stdout);
    assert!(
        delete_stdout.contains("release-captains") && delete_stdout.contains(REF_PIN_CAVEAT),
        "delete stdout must name the group and include the ref-pin caveat, got: {delete_stdout}"
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

/// AC-2: `but group` must resolve and authorize against the workspace target ref
/// (`refs/heads/main`), not the current checkout `HEAD`, when the two differ. The
/// fixture commits `administration:write` for `admin` plus a `maintainers` group
/// on `refs/heads/main`, and a different blob on branch `A` (HEAD) where `admin`
/// lacks `administration:write`. If the command read HEAD, `admin` would be
/// denied; because it reads the target ref, the add-member succeeds and the write
/// lands only in the working tree.
#[test]
#[serial_test::serial]
fn group_cli_uses_workspace_target_ref_not_head() -> anyhow::Result<()> {
    let env = head_differs_target_env()?;
    let repo = env.open_repo()?;
    let main_before = ref_id(&repo, "refs/heads/main")?;

    let add_member = env
        .but("group add-member maintainers rust-reviewer")
        .env("BUT_AGENT_HANDLE", "admin")
        .output()?;
    assert!(
        add_member.status.success(),
        "admin group add-member must authorize from refs/heads/main, not HEAD; got stderr: {}",
        String::from_utf8_lossy(&add_member.stderr)
    );
    let add_member_stdout = String::from_utf8_lossy(&add_member.stdout);
    assert!(
        add_member_stdout.contains(REF_PIN_CAVEAT),
        "add-member stdout must include the ref-pin caveat, got: {add_member_stdout}"
    );

    let worktree_permissions = std::fs::read_to_string(
        env.projects_root()
            .join(".gitbutler")
            .join("permissions.toml"),
    )?;
    let maintainers_block = group_block(&worktree_permissions, "maintainers")?;
    assert!(
        maintainers_block.contains("rust-reviewer"),
        "working-tree maintainers group must include rust-reviewer, got: {maintainers_block}"
    );

    assert_eq!(
        ref_id(&repo, "refs/heads/main")?,
        main_before,
        "CLI group add-member must not move refs/heads/main — the write is inert until committed"
    );

    Ok(())
}

/// AC-3: A feature-HEAD self-grant of `administration:write` must NOT authorize
/// a group write when the workspace target ref (`refs/heads/main`) lacks the
/// grant. The fixture commits only `reviews:read` for `rust-reviewer` on main,
/// while branch `A` (HEAD) adds a self-grant of `administration:write`. The
/// command must be denied `perm.denied` and the working-tree file must be
/// untouched.
#[test]
#[serial_test::serial]
fn group_cli_denies_using_workspace_target_when_head_self_grants() -> anyhow::Result<()> {
    let env = head_self_grants_env()?;
    let permissions_path = env
        .projects_root()
        .join(".gitbutler")
        .join("permissions.toml");
    let before = std::fs::read_to_string(&permissions_path)?;

    let denied = env
        .but("group grant maintainers reviews:write")
        .env("BUT_AGENT_HANDLE", "rust-reviewer")
        .output()?;
    assert_eq!(
        denied.status.code(),
        Some(1),
        "rust-reviewer self-grant on HEAD must not authorize group grant from target ref"
    );

    let envelope = parse_cli_error_envelope(&denied, "group grant from HEAD self-grant");
    assert_eq!(
        envelope.code, "perm.denied",
        "denial must use the stable perm.denied code"
    );
    assert!(
        envelope.message.contains("administration:write"),
        "denial message must name the missing administration:write authority, got: {}",
        envelope.message
    );
    assert!(
        !envelope.remediation_hint.trim().is_empty(),
        "denial must include a non-empty remediation_hint"
    );

    let after = std::fs::read_to_string(&permissions_path)?;
    assert_eq!(
        before, after,
        "denied group grant must not mutate the working-tree permissions.toml"
    );

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

/// Fixture where checkout HEAD is branch `A` while the workspace target ref
/// remains `refs/heads/main`. Main commits `administration:write` for `admin`
/// plus a `maintainers` group; branch `A` (HEAD) commits a different blob where
/// `admin` holds only `contents:write`. A command that mistakenly read HEAD
/// would deny `admin`.
fn head_differs_target_env() -> anyhow::Result<Sandbox> {
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
id = "rust-reviewer"
permissions = ["contents:read"]

[[group]]
name = "maintainers"
permissions = ["reviews:write"]
members = ["rust-implementer"]
EOF
cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF
git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "main governance (admin holds administration:write)"
git checkout A
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "admin"
permissions = ["contents:write"]

[[principal]]
id = "rust-reviewer"
permissions = ["contents:read"]

[[group]]
name = "maintainers"
permissions = ["reviews:write"]
members = ["rust-implementer"]
EOF
cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF
git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "feature head governance (admin lacks administration:write)"
"#,
    );
    env.but("setup").assert().success();
    Ok(env)
}

/// Fixture where `refs/heads/main` commits only `reviews:read` for
/// `rust-reviewer`, while branch `A` (HEAD) adds a self-grant of
/// `administration:write` for `rust-reviewer`. Correct target-ref authorization
/// must deny `rust-reviewer`; reading HEAD would let the self-grant through.
///
/// After `but setup` the working-tree file is re-seeded with branch A's
/// self-grant blob so the `before == after` assertion has real teeth — a buggy
/// implementation reading HEAD would authorize, and a denied command must not
/// mutate the file.
fn head_self_grants_env() -> anyhow::Result<Sandbox> {
    let env = Sandbox::open_scenario_with_target_and_default_settings("one-stack")?;
    env.invoke_bash(
        r#"
git branch -f main origin/main
git checkout main
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "admin"
permissions = ["administration:write"]

[[principal]]
id = "rust-reviewer"
permissions = ["contents:read"]

[[group]]
name = "maintainers"
permissions = ["reviews:write"]
members = ["rust-implementer"]
EOF
cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF
git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "main governance (rust-reviewer lacks administration:write)"
git checkout A
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "admin"
permissions = ["administration:write"]

[[principal]]
id = "rust-reviewer"
permissions = ["contents:read", "administration:write"]

[[group]]
name = "maintainers"
permissions = ["reviews:write"]
members = ["rust-implementer"]
EOF
cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF
git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "feature head self-grants rust-reviewer administration:write"
"#,
    );
    env.but("setup").assert().success();
    // Re-seed the working-tree file with branch A's self-grant blob so the
    // before/after assertion in AC-3 has real teeth. `but setup` may alter the
    // working-tree checkout; the authz gate reads committed target-ref config,
    // so restoring the working-tree file does not affect authorization.
    env.invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "admin"
permissions = ["administration:write"]

[[principal]]
id = "rust-reviewer"
permissions = ["contents:read", "administration:write"]

[[group]]
name = "maintainers"
permissions = ["reviews:write"]
members = ["rust-implementer"]
EOF
"#,
    );
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
