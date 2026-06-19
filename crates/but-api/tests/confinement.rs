use but_api::legacy::config_mutate::{
    AdminWriteGateError, classify_error as classify_admin_error, enforce_administration_write_gate,
};
use but_db::ForgeReview;
use serde::Serialize;

const MAIN_REF: &str = "refs/heads/main";
const FEAT_REF: &str = "refs/heads/feat";
const REVIEW_ID: usize = 1;

#[tokio::test]
#[serial_test::serial]
async fn confinement_reviewer_denied_other_merge_and_self_grant() -> anyhow::Result<()> {
    let (merge_repo, _merge_tmp) = confined_merge_repo()?;
    let main_before = ref_id(&merge_repo, MAIN_REF)?;
    let head = ref_id(&merge_repo, FEAT_REF)?;
    let ctx = context_with_review(&merge_repo, head)?;
    approve_branch(&ctx, "reviewer").await?;

    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("reviewer"))], async {
        let denial = assert_merge_denied(
            but_api::legacy::forge::merge_review(ctx.to_sync(), REVIEW_ID, None).await,
            "perm.denied",
        );
        assert!(
            denial.message.contains("merge"),
            "merge denial must name the missing merge authority, got: {}",
            denial.message
        );
        Ok::<(), anyhow::Error>(())
    })
    .await?;
    assert_eq!(
        ref_id(&merge_repo, MAIN_REF)?,
        main_before,
        "denied reviewer merge must leave the target ref unchanged"
    );

    let (admin_repo, _admin_tmp) = admin_write_confined_repo();
    let principal_id = temp_env::with_var("BUT_AGENT_HANDLE", Some("reviewer"), || {
        resolved_principal_id(&admin_repo, MAIN_REF)
    })?;
    assert_eq!(
        principal_id, "reviewer",
        "administration guard fixture must resolve the acting principal from BUT_AGENT_HANDLE"
    );

    let admin_error = temp_env::with_var("BUT_AGENT_HANDLE", Some("reviewer"), || {
        classified_admin_error(enforce_administration_write_gate(&admin_repo, MAIN_REF))
    })?;
    assert_eq!(
        admin_error.code, "perm.denied",
        "reviewer must be denied before reaching any governed config write"
    );
    assert!(
        admin_error.message.contains("administration:write"),
        "admin-write denial must name administration:write, got: {}",
        admin_error.message
    );

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn confinement_authority_from_config_not_claim() -> anyhow::Result<()> {
    let (admin_repo, _admin_tmp) = admin_write_confined_repo();
    temp_env::with_var("BUT_AGENT_HANDLE", Some("reviewer"), || {
        let cfg = but_authz::load_governance_config(&admin_repo, MAIN_REF)?;
        let principal = but_authz::resolve_principal_from_env(&cfg)?;
        assert_eq!(
            principal.id().as_str(),
            "reviewer",
            "resolver must bind the acting principal to BUT_AGENT_HANDLE"
        );
        assert!(
            principal
                .authorities()
                .contains(but_authz::Authority::ReviewsWrite),
            "reviewer must hold reviews:write from committed config"
        );
        assert!(
            !principal
                .authorities()
                .contains(but_authz::Authority::Merge),
            "reviewer must not hold merge from committed config"
        );
        let denial = but_authz::authorize(&principal, but_authz::Authority::Merge, &cfg)
            .expect_err("reviewer must not receive merge from an in-memory claim");
        assert_eq!(
            denial.code, "perm.denied",
            "direct authorization must deny missing merge"
        );
        assert!(
            denial.message.contains("merge"),
            "direct denial must name merge, got: {}",
            denial.message
        );
        Ok::<(), anyhow::Error>(())
    })?;

    let (merge_repo, _merge_tmp) = confined_merge_repo()?;
    let main_before = ref_id(&merge_repo, MAIN_REF)?;
    let head = ref_id(&merge_repo, FEAT_REF)?;
    let ctx = context_with_review(&merge_repo, head)?;
    approve_branch(&ctx, "reviewer").await?;

    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("reviewer"))], async {
        let denial = assert_merge_denied(
            but_api::legacy::forge::merge_review(ctx.to_sync(), REVIEW_ID, None).await,
            "perm.denied",
        );
        assert!(
            denial.message.contains("merge"),
            "governed merge denial must name merge, got: {}",
            denial.message
        );
        Ok::<(), anyhow::Error>(())
    })
    .await?;
    assert_eq!(
        ref_id(&merge_repo, MAIN_REF)?,
        main_before,
        "denied merge must leave the target ref unchanged"
    );

    Ok(())
}

fn confined_merge_repo() -> anyhow::Result<(gix::Repository, tempfile::TempDir)> {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
git remote add origin https://github.com/gitbutler/merge-gate-fixture.git
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "impl"
permissions = ["contents:write", "pull_requests:write", "reviews:write"]

[[principal]]
id = "reviewer"
permissions = ["reviews:write"]

[[principal]]
id = "maint"
permissions = ["merge"]

[[principal]]
id = "admin"
permissions = ["administration:write", "merge"]
EOF
cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true

[[gate]]
branch = "main"
type = "review"
min_approvals = 1
require_distinct_from_author = true
EOF
git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "governance config"
git checkout -b feat
: >.gitbutler/gates.toml
echo feat >feat.txt
git add .gitbutler/gates.toml feat.txt
git commit -m "feat"
git checkout main
"#,
        &repo,
    );
    Ok((repo, tmp))
}

fn admin_write_confined_repo() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "reviewer"
permissions = ["reviews:write"]

[[principal]]
id = "maint"
permissions = ["merge"]

[[principal]]
id = "admin"
permissions = ["administration:write", "merge"]
EOF
cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF
git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "admin write governance config"
"#,
        &repo,
    );
    (repo, tmp)
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
            title: "Confinement fixture".to_owned(),
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

async fn approve_branch(ctx: &but_ctx::Context, principal_id: &str) -> anyhow::Result<()> {
    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some(principal_id))], async {
        but_api::legacy::forge::approve_review(ctx.to_sync(), "feat".to_owned()).await
    })
    .await
}

fn resolved_principal_id(repo: &gix::Repository, target_ref: &str) -> anyhow::Result<String> {
    let cfg = but_authz::load_governance_config(repo, target_ref)?;
    let principal = but_authz::resolve_principal_from_env(&cfg)?;
    Ok(principal.id().as_str().to_owned())
}

fn fixed_time(seconds: i64) -> chrono::NaiveDateTime {
    chrono::DateTime::from_timestamp(1_735_689_600 + seconds, 0)
        .expect("fixed timestamp is valid")
        .naive_utc()
}

fn ref_id(repo: &gix::Repository, ref_name: &str) -> anyhow::Result<gix::ObjectId> {
    Ok(repo.find_reference(ref_name)?.peel_to_id()?.detach())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct GateErrorPayload {
    code: &'static str,
    message: String,
    unmet: Vec<String>,
}

fn classify_merge_error(err: &anyhow::Error) -> Option<GateErrorPayload> {
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

fn assert_merge_denied(result: anyhow::Result<()>, code: &'static str) -> GateErrorPayload {
    match result {
        Ok(()) => panic!("merge should be denied with {code}"),
        Err(err) => {
            let gate_error =
                classify_merge_error(&err).expect("merge gate errors should be structured");
            assert_eq!(
                gate_error.code, code,
                "merge gate should return the expected stable code"
            );
            gate_error
        }
    }
}

fn classified_admin_error(result: anyhow::Result<()>) -> anyhow::Result<AdminWriteGateError> {
    match result {
        Ok(()) => anyhow::bail!("administration write gate should reject this scenario"),
        Err(error) => classify_admin_error(&error)
            .ok_or_else(|| anyhow::anyhow!("admin-write guard error should classify")),
    }
}

fn _assert_admin_error_is_send_sync(_: &AdminWriteGateError) {}
