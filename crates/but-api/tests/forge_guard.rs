use anyhow::Context as _;

const FEAT_REF: &str = "refs/heads/feat";

#[test]
#[serial_test::serial]
fn forge_guard_gates_toml_only_repo_is_governed() -> anyhow::Result<()> {
    let (gates_repo, _gates_tmp) = gates_only_review_repo();
    let gates_ctx = but_ctx::Context::from_repo(gates_repo)?.with_memory_app_cache();
    let runtime = tokio::runtime::Runtime::new()?;

    let gates_error = match approve_feat_as(&runtime, &gates_ctx, "ro") {
        Ok(()) => anyhow::bail!("gates-only governance must not permit read-only approval"),
        Err(err) => err,
    };
    let gate_error = but_api::legacy::forge::classify_error(&gates_error)
        .context("gates-only governance failure should be structured")?;
    assert_eq!(
        gate_error.code, "perm.denied",
        "gates-only repos are governed and deny unauthorized approval"
    );
    assert_no_verdicts(&gates_ctx, "feat")?;
    println!(
        "gates-only approval denied with `{}` and no verdict was written",
        gate_error.code
    );

    let (ungoverned_repo, _ungoverned_tmp) = ungoverned_review_repo();
    let ungoverned_ctx = but_ctx::Context::from_repo(ungoverned_repo)?.with_memory_app_cache();
    let ungoverned_error = match approve_feat_as(&runtime, &ungoverned_ctx, "ro") {
        Ok(()) => anyhow::bail!("ungoverned approval should not record a local verdict"),
        Err(err) => err,
    };
    let ungoverned_gate_error = but_api::legacy::forge::classify_error(&ungoverned_error);
    assert!(
        !matches!(
            ungoverned_gate_error,
            Some(but_api::legacy::forge::ForgeGateError {
                code: "perm.denied",
                ..
            })
        ),
        "ungoverned control must not be treated as an authorization denial"
    );
    assert_no_verdicts(&ungoverned_ctx, "feat")?;
    println!("ungoverned approval was not classified as `perm.denied`");

    Ok(())
}

#[test]
#[serial_test::serial]
fn forge_guard_permissions_and_both_still_governed() -> anyhow::Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;

    let (permissions_repo, _permissions_tmp) = permissions_only_review_repo();
    let permissions_ctx = but_ctx::Context::from_repo(permissions_repo)?.with_memory_app_cache();
    let permissions_error = match approve_feat_as(&runtime, &permissions_ctx, "ro") {
        Ok(()) => anyhow::bail!("permissions-only governance must not permit read-only approval"),
        Err(err) => err,
    };
    let permissions_gate_error = but_api::legacy::forge::classify_error(&permissions_error)
        .context("permissions-only governance failure should be structured")?;
    assert_eq!(
        permissions_gate_error.code, "perm.denied",
        "permissions-only repos are governed and deny unauthorized approval"
    );
    assert_no_verdicts(&permissions_ctx, "feat")?;
    println!(
        "permissions-only approval denied with `{}` and no verdict was written",
        permissions_gate_error.code
    );

    let (both_repo, _both_tmp) = governed_review_repo();
    let both_ctx = but_ctx::Context::from_repo(both_repo)?.with_memory_app_cache();
    let both_error = match approve_feat_as(&runtime, &both_ctx, "ro") {
        Ok(()) => anyhow::bail!("both-files governance must deny read-only approval"),
        Err(err) => err,
    };
    let both_gate_error = but_api::legacy::forge::classify_error(&both_error)
        .context("both-files governance denial should be structured")?;
    assert_eq!(both_gate_error.code, "perm.denied");
    assert!(
        both_gate_error.message.contains("reviews:write"),
        "approval denial must name reviews:write"
    );
    assert_no_verdicts(&both_ctx, "feat")?;
    println!(
        "both-files approval denied with `{}` naming `reviews:write`",
        both_gate_error.code
    );

    Ok(())
}

#[test]
#[serial_test::serial]
fn forge_guard_authorizes_comments_and_records_approval() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_review_repo();
    let head_before = ref_id(&repo, FEAT_REF)?.to_string();
    let ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();
    let runtime = tokio::runtime::Runtime::new()?;

    temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("ro")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || -> anyhow::Result<()> {
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
        },
    )?;

    temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("reviewer")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || {
            runtime.block_on(but_api::legacy::forge::approve_review(
                ctx.to_sync(),
                "feat".to_owned(),
            ))
        },
    )?;

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

    // NOTE: `request_changes_review` was previously a contract stub here; LPR-003
    // implemented its real `changes_requested` write, so it is exercised by the
    // dedicated `local_review_assignments.rs` proofs. The two remaining verbs
    // (comment, close) are still stubs and must still fail closed.

    temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("reviewer")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
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

    temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("dev")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || -> anyhow::Result<()> {
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
        },
    )?;

    Ok(())
}

fn approve_feat_as(
    runtime: &tokio::runtime::Runtime,
    ctx: &but_ctx::Context,
    handle: &str,
) -> anyhow::Result<()> {
    temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some(handle)),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || {
            runtime.block_on(but_api::legacy::forge::approve_review(
                ctx.to_sync(),
                "feat".to_owned(),
            ))
        },
    )
}

fn assert_no_verdicts(ctx: &but_ctx::Context, target: &str) -> anyhow::Result<()> {
    let db = ctx.db.get_cache()?;
    let verdicts = db.local_review_verdicts().list_by_target(target)?;
    assert!(
        verdicts.is_empty(),
        "unauthorized or ungoverned approval must not write local_review_verdicts rows"
    );
    Ok(())
}

fn gates_only_review_repo() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
rm -rf .gitbutler
mkdir -p .gitbutler
cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "feat"
protected = true
EOF

git add .gitbutler/gates.toml
git commit -m "gates-only governance"
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

fn ungoverned_review_repo() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
rm -rf .gitbutler
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

fn permissions_only_review_repo() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
rm -rf .gitbutler
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "ro"
permissions = ["contents:read"]
EOF

git add .gitbutler/permissions.toml
git commit -m "permissions-only governance"
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
