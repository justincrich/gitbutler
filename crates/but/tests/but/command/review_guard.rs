use crate::utils::{CommandExt as _, Sandbox};

#[test]
fn review_guard_reviews_write_denied() -> anyhow::Result<()> {
    let env = governed_review_env()?;

    let dev = env
        .but("--format json review approve feat")
        .allow_json()
        .env("BUT_AUTHZ_ALLOW_ENV_HANDLE", "1")
        .env("BUT_AGENT_HANDLE", "dev")
        .output()?;
    assert!(
        !dev.status.success(),
        "dev lacks reviews:write, so approve must exit 1"
    );
    let dev_stderr = String::from_utf8_lossy(&dev.stderr);
    println!("AC-1 dev denial stderr: {dev_stderr}");
    assert!(
        dev_stderr.contains(r#""code":"perm.denied""#),
        "dev denial must be structured as perm.denied, got: {dev_stderr}"
    );
    assert!(
        dev_stderr.contains("reviews:write"),
        "dev denial must name reviews:write, got: {dev_stderr}"
    );

    for (label, handle) in [("unset", None), ("empty", Some(""))] {
        let mut cmd = env.but("--format json review approve feat").allow_json();
        match handle {
            Some(value) => {
                cmd = cmd
                    .env("BUT_AUTHZ_ALLOW_ENV_HANDLE", "1")
                    .env("BUT_AGENT_HANDLE", value);
            }
            None => {
                cmd = cmd
                    .env("BUT_AUTHZ_ALLOW_ENV_HANDLE", "1")
                    .env_remove("BUT_AGENT_HANDLE");
            }
        }
        let output = cmd.output()?;
        assert!(
            !output.status.success(),
            "{label} BUT_AGENT_HANDLE must fail closed"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("AC-1 {label} handle denial stderr: {stderr}");
        assert!(
            stderr.contains(r#""code":"perm.denied""#),
            "{label} handle denial must be structured as perm.denied, got: {stderr}"
        );
    }

    let denied_verdict_count = local_review_verdicts(&env, "feat")?.len();
    println!("AC-1 denied verdict count: {denied_verdict_count}");
    assert_eq!(
        denied_verdict_count, 0,
        "denied approve attempts must not write local_review_verdicts rows"
    );

    Ok(())
}

#[test]
fn review_guard_reviewer_commit_denied_review_accepted() -> anyhow::Result<()> {
    let env = governed_review_env()?;
    let head_before = env.invoke_git("rev-parse refs/heads/feat");

    env.file("reviewer-change.txt", "reviewer change");
    let commit = env
        .but("--format json commit feat -m reviewer-change")
        .allow_json()
        .env("BUT_AUTHZ_ALLOW_ENV_HANDLE", "1")
        .env("BUT_AGENT_HANDLE", "reviewer")
        .output()?;
    assert!(
        !commit.status.success(),
        "reviewer lacks contents:write, so commit must exit 1"
    );
    let commit_stderr = String::from_utf8_lossy(&commit.stderr);
    println!("AC-2 reviewer commit denial stderr: {commit_stderr}");
    assert!(
        commit_stderr.contains(r#""code":"perm.denied""#),
        "commit denial must be structured as perm.denied, got: {commit_stderr}"
    );
    assert!(
        commit_stderr.contains("contents:write"),
        "commit denial must name contents:write, got: {commit_stderr}"
    );
    assert_eq!(
        env.invoke_git("rev-parse refs/heads/feat"),
        head_before,
        "denied commit must leave feat HEAD unchanged"
    );

    env.but("--format json review approve feat")
        .allow_json()
        .env("BUT_AUTHZ_ALLOW_ENV_HANDLE", "1")
        .env("BUT_AGENT_HANDLE", "reviewer")
        .assert()
        .success();

    let verdicts = local_review_verdicts(&env, "feat")?;
    assert_eq!(
        verdicts.len(),
        1,
        "accepted approve must write one local_review_verdicts row"
    );
    let [verdict] = verdicts.as_slice() else {
        unreachable!("len asserted as one, so first verdict exists");
    };
    assert_eq!(verdict.principal_id, "reviewer");
    assert_eq!(verdict.verdict, "approved");
    assert_eq!(
        verdict.head_oid, head_before,
        "verdict must be pinned to feat's current head"
    );
    println!(
        "AC-2 verdict row: principal_id={}, verdict={}, head_oid={}",
        verdict.principal_id, verdict.verdict, verdict.head_oid
    );

    Ok(())
}

#[test]
fn review_guard_comment_comments_write() -> anyhow::Result<()> {
    let env = governed_review_env()?;

    let ro = env
        .but("--format json review comment feat -m note")
        .allow_json()
        .env("BUT_AUTHZ_ALLOW_ENV_HANDLE", "1")
        .env("BUT_AGENT_HANDLE", "ro")
        .output()?;
    assert!(
        !ro.status.success(),
        "ro lacks comments:write, so comment must exit 1"
    );
    let ro_stderr = String::from_utf8_lossy(&ro.stderr);
    println!("AC-3 ro comment denial stderr: {ro_stderr}");
    assert!(
        ro_stderr.contains(r#""code":"perm.denied""#),
        "comment denial must be structured as perm.denied, got: {ro_stderr}"
    );
    assert!(
        ro_stderr.contains("comments:write"),
        "comment denial must name comments:write, got: {ro_stderr}"
    );
    assert!(
        !ro_stderr.contains("reviews:write"),
        "comment denial must not be wired to reviews:write, got: {ro_stderr}"
    );

    let comments_before = local_review_comments(&env, "feat")?.len();
    assert_eq!(
        comments_before, 0,
        "no comment rows should exist before the reviewer comments"
    );

    env.but("--format json review comment feat -m note")
        .allow_json()
        .env("BUT_AUTHZ_ALLOW_ENV_HANDLE", "1")
        .env("BUT_AGENT_HANDLE", "reviewer")
        .assert()
        .success();

    let comments = local_review_comments(&env, "feat")?;
    assert_eq!(
        comments.len(),
        1,
        "authorized comment must write exactly one local_review_comments row"
    );
    let [comment] = comments.as_slice() else {
        unreachable!("len asserted as one, so first comment exists");
    };
    assert_eq!(comment.author_principal, "reviewer");
    assert_eq!(comment.body, "note");
    assert_eq!(comment.target, "feat");
    assert!(
        !comment.resolved,
        "a freshly posted comment must be unresolved"
    );
    assert_ne!(
        comment.thread_id, "__pr_meta__",
        "thread id must never be the reserved opener marker"
    );
    println!(
        "AC-3 reviewer comment row: author={}, body={}, thread={}",
        comment.author_principal, comment.body, comment.thread_id
    );

    Ok(())
}

fn governed_review_env() -> anyhow::Result<Sandbox> {
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

fn local_review_verdicts(
    env: &Sandbox,
    target: &str,
) -> anyhow::Result<Vec<but_db::LocalReviewVerdict>> {
    let ctx = env.context()?;
    let db = ctx.db.get_cache()?;
    Ok(db.local_review_verdicts().list_by_target(target)?)
}

fn local_review_comments(
    env: &Sandbox,
    target: &str,
) -> anyhow::Result<Vec<but_db::LocalReviewComment>> {
    let ctx = env.context()?;
    let db = ctx.db.get_cache()?;
    Ok(db.local_review_comments().list_by_target(target)?)
}
