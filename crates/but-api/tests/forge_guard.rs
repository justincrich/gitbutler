use anyhow::Context as _;

const FEAT_REF: &str = "refs/heads/feat";

#[test]
#[serial_test::serial]
fn forge_guard_authorizes_comments_and_records_approval() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_review_repo();
    let head_before = ref_id(&repo, FEAT_REF)?.to_string();
    let ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();
    let runtime = tokio::runtime::Runtime::new()?;

    temp_env::with_var("BUT_AGENT_HANDLE", Some("ro"), || -> anyhow::Result<()> {
        let err = match runtime.block_on(but_api::legacy::forge::comment_review(
            ctx.to_sync(),
            "feat".to_owned(),
            "note".to_owned(),
        )) {
            Ok(()) => anyhow::bail!("read-only principal should be denied comments:write"),
            Err(err) => err,
        };
        let gate_error = but_api::legacy::forge::classify_error(&err)
            .context("comment denial should be structured")?;
        assert_eq!(gate_error.code, "perm.denied");
        assert!(
            gate_error.message.contains("comments:write"),
            "comment denial must name comments:write"
        );
        println!("api comment denied with `perm.denied` naming `comments:write`");
        Ok(())
    })?;

    temp_env::with_var("BUT_AGENT_HANDLE", Some("reviewer"), || {
        runtime.block_on(but_api::legacy::forge::approve_review(
            ctx.to_sync(),
            "feat".to_owned(),
        ))
    })?;

    let db = ctx.db.get_cache()?;
    let verdicts = db.local_review_verdicts().list_by_target("feat")?;
    assert_eq!(
        verdicts.len(),
        1,
        "approved review must write one local_review_verdicts row"
    );
    let [verdict] = verdicts.as_slice() else {
        unreachable!("len asserted as one, so first verdict exists");
    };
    assert_eq!(verdict.principal_id, "reviewer");
    assert_eq!(verdict.verdict, "approved");
    assert_eq!(
        verdict.head_oid, head_before,
        "approved verdict must be pinned to feat head"
    );
    println!(
        "api approval verdict: principal_id={}, verdict={}, head_oid={}",
        verdict.principal_id, verdict.verdict, verdict.head_oid
    );

    Ok(())
}

#[test]
#[serial_test::serial]
fn forge_guard_no_stub_success_for_unimplemented_review_actions() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_review_repo();
    let ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();
    let runtime = tokio::runtime::Runtime::new()?;

    temp_env::with_var(
        "BUT_AGENT_HANDLE",
        Some("reviewer"),
        || -> anyhow::Result<()> {
            let err = match runtime.block_on(but_api::legacy::forge::request_changes_review(
                ctx.to_sync(),
                "feat".to_owned(),
                Some("please fix this".to_owned()),
            )) {
                Ok(()) => anyhow::bail!("request-changes must not report success without behavior"),
                Err(err) => err,
            };
            let message = err.to_string();
            assert!(
                message.contains("request_changes_review"),
                "request-changes blocker must name the unsupported action, got: {message}"
            );
            assert!(
                message.contains("no downstream"),
                "request-changes blocker must explain no downstream behavior exists, got: {message}"
            );
            println!("request_changes_review blocker: {message}");
            Ok(())
        },
    )?;

    temp_env::with_var(
        "BUT_AGENT_HANDLE",
        Some("reviewer"),
        || -> anyhow::Result<()> {
            let err = match runtime.block_on(but_api::legacy::forge::comment_review(
                ctx.to_sync(),
                "feat".to_owned(),
                "note".to_owned(),
            )) {
                Ok(()) => anyhow::bail!("comment must not report success without behavior"),
                Err(err) => err,
            };
            let message = err.to_string();
            assert!(
                message.contains("comment_review"),
                "comment blocker must name the unsupported action, got: {message}"
            );
            assert!(
                message.contains("no downstream"),
                "comment blocker must explain no downstream behavior exists, got: {message}"
            );
            println!("comment_review blocker: {message}");
            Ok(())
        },
    )?;

    temp_env::with_var("BUT_AGENT_HANDLE", Some("dev"), || -> anyhow::Result<()> {
        let err = match runtime.block_on(but_api::legacy::forge::close_review(
            ctx.to_sync(),
            "feat".to_owned(),
        )) {
            Ok(()) => anyhow::bail!("close must not report success without behavior"),
            Err(err) => err,
        };
        let message = err.to_string();
        assert!(
            message.contains("close_review"),
            "close blocker must name the unsupported action, got: {message}"
        );
        assert!(
            message.contains("no downstream"),
            "close blocker must explain no downstream behavior exists, got: {message}"
        );
        println!("close_review blocker: {message}");
        Ok(())
    })?;

    Ok(())
}

fn governed_review_repo() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "reviewer"
permissions = ["reviews:write", "comments:write", "contents:read"]

[[principal]]
id = "ro"
permissions = ["contents:read"]

[[principal]]
id = "dev"
permissions = ["contents:write", "pull_requests:write"]
EOF

cat >.gitbutler/gates.toml <<'EOF'
EOF

git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "governance config"
git checkout -b feat
echo feat-base >feat-base.txt
git add feat-base.txt
git commit -m "feat base"
git checkout main
"#,
        &repo,
    );
    (repo, tmp)
}

fn ref_id(repo: &gix::Repository, ref_name: &str) -> anyhow::Result<gix::ObjectId> {
    Ok(repo.find_reference(ref_name)?.peel_to_id()?.detach())
}
