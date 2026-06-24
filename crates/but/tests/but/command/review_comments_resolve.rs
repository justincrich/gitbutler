use crate::utils::Sandbox;

/// `but review comments <branch>` lists every non-reserved thread on the branch,
/// showing thread id, file/line, author, and resolved state.
#[test]
fn review_comments_lists_threads() -> anyhow::Result<()> {
    let env = governed_review_env_with_commenter()?;

    // Seed a branch-level comment as the reviewer (generates a fresh thread id).
    env.but("review comment feat -m 'first thread'")
        .env("BUT_AGENT_HANDLE", "reviewer")
        .assert()
        .success();

    // Seed a code comment on a specific file+line as the commenter (second thread).
    env.but("review comment feat -m 'code note' --file src/lib.rs --line 42")
        .env("BUT_AGENT_HANDLE", "commenter")
        .assert()
        .success();

    // `comments` is a read (comments:read / branch-scoped read) — any comments:write
    // holder can call it. Run as the reviewer.
    let output = env
        .but("review comments feat")
        .env("BUT_AGENT_HANDLE", "reviewer")
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "comments must exit 0, got stderr: {stderr}"
    );

    // Both seeded comments must appear with author + body.
    assert!(
        stdout.contains("first thread"),
        "human output must include first thread body, got: {stdout}"
    );
    assert!(
        stdout.contains("code note"),
        "human output must include code-note body, got: {stdout}"
    );
    assert!(
        stdout.contains("reviewer"),
        "human output must include the reviewer author, got: {stdout}"
    );
    assert!(
        stdout.contains("commenter"),
        "human output must include the commenter author, got: {stdout}"
    );
    assert!(
        stdout.contains("src/lib.rs"),
        "human output must include the file path for the code comment, got: {stdout}"
    );
    assert!(
        stdout.contains("42"),
        "human output must include the line number for the code comment, got: {stdout}"
    );

    Ok(())
}

/// `but review resolve <branch> --thread <id>` flips the thread to resolved when
/// the caller is the thread author (one of the three R22 resolver identities).
#[test]
fn review_resolve_by_thread_author() -> anyhow::Result<()> {
    let env = governed_review_env_with_commenter()?;

    // The commenter posts a comment, generating a fresh thread.
    env.but("review comment feat -m 'resolve me'")
        .env("BUT_AGENT_HANDLE", "commenter")
        .assert()
        .success();

    let comments = local_review_comments(&env, "feat")?;
    assert_eq!(comments.len(), 1, "one comment row should exist");
    let thread_id = comments[0].thread_id.clone();
    assert!(
        !comments[0].resolved,
        "freshly posted comment must be unresolved"
    );

    // The commenter (thread author) resolves it.
    env.but(format!("review resolve feat --thread {thread_id}"))
        .env("BUT_AGENT_HANDLE", "commenter")
        .assert()
        .success();

    let after = local_review_comments(&env, "feat")?;
    assert_eq!(after.len(), 1, "resolve must not create or delete rows");
    assert!(
        after[0].resolved,
        "comment must be resolved after the author resolves the thread"
    );

    Ok(())
}

/// `but review resolve` rejects a principal that holds `comments:write` but is
/// neither the thread author, the assigned reviewer, nor a `reviews:write`
/// holder (R22 resolver-identity constraint).
#[test]
fn review_resolve_rejects_non_resolver() -> anyhow::Result<()> {
    let env = governed_review_env_with_commenter()?;

    // The reviewer (reviews:write + comments:write) posts a comment.
    env.but("review comment feat -m 'owned thread'")
        .env("BUT_AGENT_HANDLE", "reviewer")
        .assert()
        .success();

    let comments = local_review_comments(&env, "feat")?;
    assert_eq!(comments.len(), 1, "one comment row should exist");
    let thread_id = comments[0].thread_id.clone();

    // The commenter has comments:write but is NOT the author, NOT an assigned
    // reviewer, and does NOT hold reviews:write → R22 denial.
    let denied = env
        .but(format!("review resolve feat --thread {thread_id}"))
        .env("BUT_AGENT_HANDLE", "commenter")
        .output()?;
    assert!(
        !denied.status.success(),
        "commenter must not be able to resolve a thread they don't own (R22)"
    );
    let denied_stderr = String::from_utf8_lossy(&denied.stderr);
    println!("R22 non-resolver denial stderr: {denied_stderr}");
    assert!(
        denied_stderr.contains(r#""code":"perm.denied""#),
        "R22 denial must be structured as perm.denied, got: {denied_stderr}"
    );
    assert!(
        denied_stderr.contains("R22"),
        "R22 denial must reference the resolver-identity constraint, got: {denied_stderr}"
    );

    // No rows must have been flipped.
    let after = local_review_comments(&env, "feat")?;
    assert_eq!(after.len(), 1, "denied resolve must not change row count");
    assert!(
        !after[0].resolved,
        "denied resolve must leave the thread unresolved"
    );

    Ok(())
}

/// A governed review env that adds a `commenter` principal with only
/// `comments:write` (no `reviews:write`) so R22 resolver-identity can be
/// tested in isolation.
fn governed_review_env_with_commenter() -> anyhow::Result<Sandbox> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack")?;
    env.invoke_bash(
        r#"
git branch -m A feat
write_governance_commit() {
    target_ref="$1"
    base=$(git rev-parse "$target_ref")
    index=$(mktemp)
    export GIT_INDEX_FILE="$index"
    git read-tree "$base"
    permissions_blob=$(git hash-object -w --stdin <<'EOF'
[[principal]]
id = "reviewer"
permissions = ["reviews:write", "comments:write", "contents:read"]

[[principal]]
id = "commenter"
permissions = ["comments:write", "contents:read"]

[[principal]]
id = "dev"
permissions = ["contents:write", "pull_requests:write"]

[[principal]]
id = "ro"
permissions = ["contents:read"]
EOF
)
    gates_blob=$(git hash-object -w --stdin <<'EOF'
EOF
)
    git update-index --add --cacheinfo 100644 "$permissions_blob" .gitbutler/permissions.toml
    git update-index --add --cacheinfo 100644 "$gates_blob" .gitbutler/gates.toml
    tree=$(git write-tree)
    commit=$(printf 'governance config\n' | git commit-tree "$tree" -p "$base")
    git update-ref "$target_ref" "$commit"
    rm "$index"
    unset GIT_INDEX_FILE
}
write_governance_commit refs/heads/main
write_governance_commit refs/heads/feat
"#,
    );
    env.but("setup").assert().success();
    env.set_target_sha("refs/heads/main")?;
    env.setup_metadata(&["feat"])?;
    Ok(env)
}

fn local_review_comments(
    env: &Sandbox,
    target: &str,
) -> anyhow::Result<Vec<but_db::LocalReviewComment>> {
    let ctx = env.context()?;
    let db = ctx.db.get_cache()?;
    Ok(db.local_review_comments().list_by_target(target)?)
}
