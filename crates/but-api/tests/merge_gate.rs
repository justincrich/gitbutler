use but_db::ForgeReview;
use serde::Serialize;

const MAIN_REF: &str = "refs/heads/main";
const FEAT_REF: &str = "refs/heads/feat";
const REVIEW_ID: usize = 1;

#[tokio::test]
#[serial_test::serial]
async fn merge_gate_self_and_stale_dismissed() -> anyhow::Result<()> {
    let (repo, _tmp) = merge_gated_repo(GateConfig::Single)?;
    let main_before = ref_id(&repo, MAIN_REF)?;
    let head = ref_id(&repo, FEAT_REF)?;
    let ctx = context_with_review(&repo, head)?;

    approve_branch(&ctx, "impl").await?;
    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("maint"))], async {
        let denial = assert_gate_denied(
            but_api::legacy::forge::merge_review(ctx.to_sync(), REVIEW_ID, None).await,
            "gate.review_required",
        );
        assert!(
            denial.unmet.iter().any(|entry| entry == "no_approval"),
            "self-approval denial should report the no_approval discriminator"
        );
        Ok::<(), anyhow::Error>(())
    })
    .await?;
    assert_eq!(
        ref_id(&repo, MAIN_REF)?,
        main_before,
        "self-approval denial must leave main unchanged"
    );

    let (repo, _tmp) = merge_gated_repo(GateConfig::Single)?;
    let head_h1 = ref_id(&repo, FEAT_REF)?;
    let ctx = context_with_review(&repo, head_h1)?;

    approve_branch(&ctx, "reviewer").await?;
    advance_feature_head(&repo)?;
    let head_h2 = ref_id(&repo, FEAT_REF)?;
    assert_ne!(
        head_h1, head_h2,
        "stale-approval fixture must advance feat from H1 to H2"
    );

    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("maint"))], async {
        let denial = assert_gate_denied(
            but_api::legacy::forge::merge_review(ctx.to_sync(), REVIEW_ID, None).await,
            "gate.review_required",
        );
        assert!(
            denial
                .unmet
                .iter()
                .any(|entry| entry == "approval_stale_at_head"),
            "stale approval denial should report approval_stale_at_head"
        );
        Ok::<(), anyhow::Error>(())
    })
    .await?;

    approve_branch(&ctx, "reviewer").await?;
    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("maint"))], async {
        let err = but_api::legacy::forge::merge_review(ctx.to_sync(), REVIEW_ID, None)
            .await
            .expect_err("local fixture should reach the forge call and fail outside governance");
        assert!(
            classify_error(&err).is_none(),
            "fresh re-approval at H2 should satisfy the merge gate"
        );
        Ok::<(), anyhow::Error>(())
    })
    .await?;

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn merge_gate_distinct_current_head_satisfies() -> anyhow::Result<()> {
    let (repo, _tmp) = merge_gated_repo(GateConfig::Single)?;
    let head = ref_id(&repo, FEAT_REF)?;
    let ctx = context_with_review(&repo, head)?;

    approve_branch(&ctx, "reviewer").await?;

    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("maint"))], async {
        let err = but_api::legacy::forge::merge_review(ctx.to_sync(), REVIEW_ID, None)
            .await
            .expect_err("local fixture should reach the forge call and fail outside governance");
        assert!(
            classify_error(&err).is_none(),
            "distinct current-head approval should satisfy the merge gate"
        );
        Ok::<(), anyhow::Error>(())
    })
    .await?;

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn merge_gate_authorize_and_review_requirement() -> anyhow::Result<()> {
    let (repo, _tmp) = merge_gated_repo(GateConfig::Single)?;
    let main_before = ref_id(&repo, MAIN_REF)?;
    let head = ref_id(&repo, FEAT_REF)?;
    let ctx = context_with_review(&repo, head)?;
    approve_branch(&ctx, "reviewer").await?;

    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("impl"))], async {
        let denial = assert_gate_denied(
            but_api::legacy::forge::merge_review(ctx.to_sync(), REVIEW_ID, None).await,
            "perm.denied",
        );
        assert!(
            denial.message.contains("merge"),
            "perm.denied should name the missing merge authority"
        );

        let denial = assert_gate_denied(
            but_api::legacy::forge::set_review_auto_merge(ctx.to_sync(), REVIEW_ID, true).await,
            "perm.denied",
        );
        assert!(
            denial.message.contains("merge"),
            "auto-merge denial should name the missing merge authority"
        );

        anyhow::Ok(())
    })
    .await?;

    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("maint"))], async {
        let err = but_api::legacy::forge::merge_review(ctx.to_sync(), REVIEW_ID, None)
            .await
            .expect_err("local fixture should reach the forge call and fail outside governance");
        assert!(
            classify_error(&err).is_none(),
            "authorized reviewed merge should not fail with a governance denial"
        );
        Ok::<(), anyhow::Error>(())
    })
    .await?;

    assert_eq!(
        ref_id(&repo, MAIN_REF)?,
        main_before,
        "merge gate tests must not mutate the local target ref"
    );

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn merge_gate_below_min_approvals_blocked() -> anyhow::Result<()> {
    let (repo, _tmp) = merge_gated_repo(GateConfig::Single)?;
    let main_before = ref_id(&repo, MAIN_REF)?;
    let ctx = context_with_review(&repo, ref_id(&repo, FEAT_REF)?)?;

    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("maint"))], async {
        let denial = assert_gate_denied(
            but_api::legacy::forge::merge_review(ctx.to_sync(), REVIEW_ID, None).await,
            "gate.review_required",
        );
        assert!(
            denial.message.contains("min_approvals") || denial.message.contains("approval"),
            "gate.review_required should name the unmet approval shortfall"
        );
        Ok::<(), anyhow::Error>(())
    })
    .await?;

    assert_eq!(
        ref_id(&repo, MAIN_REF)?,
        main_before,
        "below-min-approval denial must leave main unchanged"
    );

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn merge_gate_two_group_both_present_proceeds() -> anyhow::Result<()> {
    let (repo, _tmp) = merge_gated_repo(GateConfig::TwoGroup)?;
    let head = ref_id(&repo, FEAT_REF)?;
    let ctx = context_with_review(&repo, head)?;

    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("maint"))], async {
        let denial = assert_gate_denied(
            but_api::legacy::forge::merge_review(ctx.to_sync(), REVIEW_ID, None).await,
            "gate.review_required",
        );
        assert!(
            denial.message.contains("code-reviewers") && denial.message.contains("maintainers"),
            "two-group denial should name both missing approval groups"
        );
        Ok::<(), anyhow::Error>(())
    })
    .await?;

    approve_branch(&ctx, "reviewer-a").await?;
    approve_branch(&ctx, "reviewer-b").await?;

    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("maint"))], async {
        let err = but_api::legacy::forge::merge_review(ctx.to_sync(), REVIEW_ID, None)
            .await
            .expect_err("local fixture should reach the forge call and fail outside governance");
        assert!(
            classify_error(&err).is_none(),
            "both required groups approving at head should satisfy the merge gate"
        );
        Ok::<(), anyhow::Error>(())
    })
    .await?;

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn merge_gate_dryrun_and_malformed_failclosed() -> anyhow::Result<()> {
    let (repo, _tmp) = merge_gated_repo(GateConfig::Single)?;
    let main_before = ref_id(&repo, MAIN_REF)?;
    let ctx = context_with_review(&repo, ref_id(&repo, FEAT_REF)?)?;
    approve_branch(&ctx, "reviewer").await?;
    let verdicts_before = verdict_count(&ctx)?;

    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("impl"))], async {
        let denial = assert_gate_denied(
            but_api::legacy::forge::merge_review(ctx.to_sync(), REVIEW_ID, None).await,
            "perm.denied",
        );
        assert!(
            denial.message.contains("merge"),
            "dry-run-equivalent merge path should still require merge authority"
        );
        Ok::<(), anyhow::Error>(())
    })
    .await?;

    assert_eq!(
        ref_id(&repo, MAIN_REF)?,
        main_before,
        "denied dry-run-equivalent merge must leave main unchanged"
    );
    assert_eq!(
        verdict_count(&ctx)?,
        verdicts_before,
        "denied merge must not mutate local review verdicts"
    );

    let (repo, _tmp) = merge_gated_repo(GateConfig::Malformed)?;
    let ctx = context_with_review(&repo, ref_id(&repo, FEAT_REF)?)?;

    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("maint"))], async {
        assert_gate_denied(
            but_api::legacy::forge::merge_review(ctx.to_sync(), REVIEW_ID, None).await,
            "config.invalid",
        );
        Ok::<(), anyhow::Error>(())
    })
    .await?;

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn merge_gate_unknown_and_no_handle_failclosed() -> anyhow::Result<()> {
    let (repo, _tmp) = merge_gated_repo(GateConfig::Single)?;
    let main_before = ref_id(&repo, MAIN_REF)?;
    let ctx = context_with_review(&repo, ref_id(&repo, FEAT_REF)?)?;
    approve_branch(&ctx, "reviewer").await?;

    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("ghost"))], async {
        let denial = assert_gate_denied(
            but_api::legacy::forge::merge_review(ctx.to_sync(), REVIEW_ID, None).await,
            "perm.denied",
        );
        assert!(
            denial
                .message
                .contains("principal \"ghost\" not found in committed governance config"),
            "unknown principal denial should name the missing handle"
        );
        assert!(
            denial.unmet.is_empty(),
            "perm.denied should not carry review-requirement unmet entries"
        );
        Ok::<(), anyhow::Error>(())
    })
    .await?;
    assert_eq!(
        ref_id(&repo, MAIN_REF)?,
        main_before,
        "unknown-principal denial must leave main unchanged"
    );

    temp_env::async_with_vars([("BUT_AGENT_HANDLE", None::<&str>)], async {
        let denial = assert_gate_denied(
            but_api::legacy::forge::merge_review(ctx.to_sync(), REVIEW_ID, None).await,
            "perm.denied",
        );
        assert!(
            denial
                .message
                .contains("BUT_AGENT_HANDLE is required to resolve a governed principal"),
            "no-handle denial should name BUT_AGENT_HANDLE"
        );
        assert!(
            denial.unmet.is_empty(),
            "perm.denied should not carry review-requirement unmet entries"
        );
        Ok::<(), anyhow::Error>(())
    })
    .await?;
    assert_eq!(
        ref_id(&repo, MAIN_REF)?,
        main_before,
        "no-handle denial must leave main unchanged"
    );

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn merge_gate_malformed_config_is_config_invalid() -> anyhow::Result<()> {
    let (repo, _tmp) = merge_gated_repo(GateConfig::Malformed)?;
    let main_before = ref_id(&repo, MAIN_REF)?;
    let ctx = context_with_review(&repo, ref_id(&repo, FEAT_REF)?)?;

    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("maint"))], async {
        let denial = assert_gate_denied(
            but_api::legacy::forge::merge_review(ctx.to_sync(), REVIEW_ID, None).await,
            "config.invalid",
        );
        assert!(
            denial.message.contains(".gitbutler/gates.toml"),
            "malformed target-ref gate config should be identified"
        );
        Ok::<(), anyhow::Error>(())
    })
    .await?;
    assert_eq!(
        ref_id(&repo, MAIN_REF)?,
        main_before,
        "malformed-config denial for maint must leave main unchanged"
    );

    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("ghost"))], async {
        let denial = assert_gate_denied(
            but_api::legacy::forge::merge_review(ctx.to_sync(), REVIEW_ID, None).await,
            "config.invalid",
        );
        assert!(
            denial.message.contains(".gitbutler/gates.toml"),
            "malformed target-ref gate config should win before principal resolution"
        );
        Ok::<(), anyhow::Error>(())
    })
    .await?;
    assert_eq!(
        ref_id(&repo, MAIN_REF)?,
        main_before,
        "malformed-config denial for ghost must leave main unchanged"
    );

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn merge_gate_undefined_required_group_denied() -> anyhow::Result<()> {
    let (repo, _tmp) = merge_gated_repo(GateConfig::UndefinedGroup)?;
    let main_before = ref_id(&repo, MAIN_REF)?;
    let ctx = context_with_review(&repo, ref_id(&repo, FEAT_REF)?)?;
    approve_branch(&ctx, "reviewer").await?;

    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("maint"))], async {
        let denial = assert_gate_denied(
            but_api::legacy::forge::merge_review(ctx.to_sync(), REVIEW_ID, None).await,
            "gate.review_required",
        );
        assert!(
            denial
                .unmet
                .iter()
                .any(|entry| entry == "undefined required group ghost-reviewers"),
            "undefined required group must be reported as unsatisfiable, got {:?}",
            denial.unmet
        );
        assert!(
            denial.message.contains("ghost-reviewers"),
            "undefined group denial should name the required group"
        );
        Ok::<(), anyhow::Error>(())
    })
    .await?;
    assert_eq!(
        ref_id(&repo, MAIN_REF)?,
        main_before,
        "undefined-required-group denial must leave main unchanged"
    );

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn merge_gate_dryrun_unknown_failclosed_persists_nothing() -> anyhow::Result<()> {
    let (repo, _tmp) = merge_gated_repo(GateConfig::Single)?;
    let main_before = ref_id(&repo, MAIN_REF)?;
    let ctx = context_with_review(&repo, ref_id(&repo, FEAT_REF)?)?;
    approve_branch(&ctx, "reviewer").await?;
    let verdicts_before = verdict_count(&ctx)?;

    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("ghost"))], async {
        let denial = assert_gate_denied(
            but_api::legacy::forge::dry_run_merge_review(ctx.to_sync(), REVIEW_ID),
            "perm.denied",
        );
        assert!(
            denial
                .message
                .contains("principal \"ghost\" not found in committed governance config"),
            "dry-run unknown principal denial should name the missing handle"
        );
        Ok::<(), anyhow::Error>(())
    })
    .await?;

    assert_eq!(
        ref_id(&repo, MAIN_REF)?,
        main_before,
        "denied dry run must leave main unchanged"
    );
    assert_eq!(
        verdict_count(&ctx)?,
        verdicts_before,
        "denied dry run must not mutate local review verdicts"
    );

    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum GateConfig {
    Single,
    TwoGroup,
    Malformed,
    UndefinedGroup,
}

fn merge_gated_repo(config: GateConfig) -> anyhow::Result<(gix::Repository, tempfile::TempDir)> {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    let gates = match config {
        GateConfig::Single => {
            r#"
[[branch]]
name = "main"
protected = true

[[gate]]
branch = "main"
type = "review"
min_approvals = 1
require_distinct_from_author = true
"#
        }
        GateConfig::TwoGroup => {
            r#"
[[branch]]
name = "main"
protected = true

[[gate]]
branch = "main"
type = "review"
min_approvals = 1
require_distinct_from_author = true
require_approval_from_group = ["code-reviewers", "maintainers"]
"#
        }
        GateConfig::Malformed => {
            r#"
[[branch]
name = "main"
protected = nope
"#
        }
        GateConfig::UndefinedGroup => {
            r#"
[[branch]]
name = "main"
protected = true

[[gate]]
branch = "main"
type = "review"
min_approvals = 1
require_distinct_from_author = true
require_approval_from_group = ["ghost-reviewers"]
"#
        }
    };

    let permissions = match config {
        GateConfig::TwoGroup => {
            r#"
[[principal]]
id = "impl"
permissions = ["contents:write", "pull_requests:write", "reviews:write"]

[[principal]]
id = "reviewer-a"
permissions = ["reviews:write"]
groups = ["code-reviewers"]

[[principal]]
id = "reviewer-b"
permissions = ["reviews:write"]
groups = ["maintainers"]

[[principal]]
id = "maint"
permissions = ["merge", "reviews:write"]
groups = ["maintainers"]

[[group]]
name = "code-reviewers"
permissions = ["reviews:write"]
members = ["reviewer-a"]

[[group]]
name = "maintainers"
permissions = ["merge", "reviews:write"]
members = ["reviewer-b", "maint"]
"#
        }
        GateConfig::Single | GateConfig::Malformed | GateConfig::UndefinedGroup => {
            r#"
[[principal]]
id = "impl"
permissions = ["contents:write", "pull_requests:write", "reviews:write"]

[[principal]]
id = "reviewer"
permissions = ["reviews:write"]

[[principal]]
id = "maint"
permissions = ["merge"]
"#
        }
    };

    but_testsupport::invoke_bash(
        &format!(
            r#"
git remote add origin https://github.com/gitbutler/merge-gate-fixture.git
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
{permissions}
EOF
cat >.gitbutler/gates.toml <<'EOF'
{gates}
EOF
git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "governance config"
git checkout -b feat
: >.gitbutler/gates.toml
echo feat >feat.txt
git add .gitbutler/gates.toml feat.txt
git commit -m "feat"
git checkout main
"#
        ),
        &repo,
    );

    Ok((repo, tmp))
}

fn context_with_review(
    repo: &gix::Repository,
    head: gix::ObjectId,
) -> anyhow::Result<but_ctx::Context> {
    let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
    seed_review(&mut ctx, head)?;
    Ok(ctx)
}

fn seed_review(ctx: &mut but_ctx::Context, head: gix::ObjectId) -> anyhow::Result<()> {
    ctx.db
        .get_cache_mut()?
        .forge_reviews_mut()?
        .upsert(ForgeReview {
            html_url: "https://github.com/gitbutler/merge-gate-fixture/pull/1".to_owned(),
            number: REVIEW_ID.try_into()?,
            title: "Merge gate fixture".to_owned(),
            body: None,
            author: Some("impl".to_owned()),
            labels: "[]".to_owned(),
            draft: false,
            source_branch: "feat".to_owned(),
            target_branch: "main".to_owned(),
            sha: head.to_string(),
            created_at: None,
            modified_at: None,
            merged_at: None,
            closed_at: None,
            repository_ssh_url: None,
            repository_https_url: Some(
                "https://github.com/gitbutler/merge-gate-fixture.git".to_owned(),
            ),
            repo_owner: Some("gitbutler".to_owned()),
            head_repo_is_fork: false,
            reviewers: "[]".to_owned(),
            unit_symbol: "#".to_owned(),
            last_sync_at: fixed_time(0),
            struct_version: but_forge::ForgeReview::struct_version(),
        })?;
    Ok(())
}

fn fixed_time(seconds: i64) -> chrono::NaiveDateTime {
    chrono::DateTime::from_timestamp(1_735_689_600 + seconds, 0)
        .expect("fixed timestamp is valid")
        .naive_utc()
}

fn ref_id(repo: &gix::Repository, ref_name: &str) -> anyhow::Result<gix::ObjectId> {
    Ok(repo.find_reference(ref_name)?.peel_to_id()?.detach())
}

fn verdict_count(ctx: &but_ctx::Context) -> anyhow::Result<usize> {
    Ok(ctx
        .db
        .get_cache()?
        .local_review_verdicts()
        .list_by_target(FEAT_REF)?
        .len())
}

async fn approve_branch(ctx: &but_ctx::Context, principal_id: &str) -> anyhow::Result<()> {
    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some(principal_id))], async {
        but_api::legacy::forge::approve_review(ctx.to_sync(), "feat".to_owned()).await
    })
    .await
}

fn advance_feature_head(repo: &gix::Repository) -> anyhow::Result<()> {
    but_testsupport::invoke_bash(
        r#"
git checkout feat
echo H2 >>feat.txt
git add feat.txt
git commit -m "advance feat"
git checkout main
"#,
        repo,
    );
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct GateErrorPayload {
    code: &'static str,
    message: String,
    unmet: Vec<String>,
}

fn classify_error(err: &anyhow::Error) -> Option<GateErrorPayload> {
    if let Some(error) = err.downcast_ref::<but_api::legacy::merge_gate::MergeGateError>() {
        return Some(GateErrorPayload {
            code: error.code,
            message: error.message.clone(),
            unmet: error.unmet.clone(),
        });
    }

    if let Some(denial) = err.downcast_ref::<but_authz::Denial>() {
        return Some(GateErrorPayload {
            code: denial.code,
            message: denial.message.clone(),
            unmet: Vec::new(),
        });
    }

    err.downcast_ref::<but_authz::ConfigError>()
        .map(|error| GateErrorPayload {
            code: error.code(),
            message: error.to_string(),
            unmet: Vec::new(),
        })
}

fn assert_gate_denied(result: anyhow::Result<()>, code: &'static str) -> GateErrorPayload {
    match result {
        Ok(()) => panic!("merge should be denied with {code}"),
        Err(err) => {
            let gate_error = classify_error(&err).expect("merge gate errors should be structured");
            assert_eq!(
                gate_error.code, code,
                "merge gate should return the expected stable code"
            );
            gate_error
        }
    }
}
