use but_core::RefMetadata as _;
use but_db::ForgeReview;
use gix::refs::FullName;

use crate::utils::{CommandExt as _, Sandbox};

const REVIEW_ID: usize = 1;

#[test]
#[serial_test::serial]
fn confinement_no_inband_identity_override() -> anyhow::Result<()> {
    let env = confined_review_env()?;

    env.but("--format json review approve A")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "reviewer")
        .assert()
        .success();
    let verdicts = local_review_verdicts(&env, "A")?;
    let [verdict] = verdicts.as_slice() else {
        anyhow::bail!("reviewer approval should write exactly one local verdict");
    };
    assert_eq!(
        verdict.principal_id, "reviewer",
        "review approve must bind the acting principal to BUT_AGENT_HANDLE"
    );

    let denied = env
        .but("--format json pr merge 1 --dry-run")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "reviewer")
        .output()?;
    assert!(
        !denied.status.success(),
        "reviewer merge without merge authority must fail"
    );
    let stderr = String::from_utf8_lossy(&denied.stderr);
    assert!(
        stderr.contains(r#""code":"perm.denied""#),
        "merge denial must be structured as perm.denied, got: {stderr}"
    );
    assert!(
        stderr.contains("merge"),
        "merge denial must name the missing merge authority, got: {stderr}"
    );

    let override_attempt = env
        .but("--format json pr merge 1 --dry-run --as maint")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "reviewer")
        .output()?;
    assert!(
        !override_attempt.status.success(),
        "`--as maint` must not be accepted as an identity override"
    );
    let override_stderr = String::from_utf8_lossy(&override_attempt.stderr);
    assert!(
        override_stderr.contains("--as"),
        "clap rejection must name the unsupported --as argument, got: {override_stderr}"
    );
    assert!(
        !override_stderr.contains("forge merge_review boundary rejected review"),
        "unsupported --as argument must stop before running the governed action, got: {override_stderr}"
    );

    Ok(())
}

fn confined_review_env() -> anyhow::Result<Sandbox> {
    let env = Sandbox::open_scenario_with_target_and_default_settings("one-stack")?;
    env.invoke_bash(
        r#"
git remote set-url origin https://github.com/gitbutler/merge-gate-fixture.git
git branch -f main origin/main
git checkout main
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "impl"
permissions = ["contents:write", "pull_requests:write"]

[[principal]]
id = "reviewer"
permissions = ["reviews:write"]

[[principal]]
id = "maint"
permissions = ["merge"]
EOF
cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF
git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "governance config"
git checkout A
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "impl"
permissions = ["contents:write", "pull_requests:write"]

[[principal]]
id = "reviewer"
permissions = ["reviews:write"]

[[principal]]
id = "maint"
permissions = ["merge"]
EOF
cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true

[[branch]]
name = "A"
protected = true
EOF
git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "branch governance config"
"#,
    );
    env.but("setup").assert().success();
    env.setup_metadata(&["A"])?;
    attach_review_id(&env, "A", REVIEW_ID)?;
    seed_review_cache(&env)?;
    Ok(env)
}

fn attach_review_id(env: &Sandbox, branch_name: &str, review_id: usize) -> anyhow::Result<()> {
    let mut meta = env.meta()?;
    let ref_name = FullName::try_from(format!("refs/heads/{branch_name}"))?;
    let mut branch = meta.branch(ref_name.as_ref())?;
    branch.review.pull_request = Some(review_id);
    meta.set_branch(&branch)?;
    Ok(())
}

fn seed_review_cache(env: &Sandbox) -> anyhow::Result<()> {
    let repo = env.open_repo()?;
    let head = ref_id(&repo, "refs/heads/A")?;
    let ctx = env.context()?;
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
            source_branch: "A".to_owned(),
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

fn local_review_verdicts(
    env: &Sandbox,
    target: &str,
) -> anyhow::Result<Vec<but_db::LocalReviewVerdict>> {
    let ctx = env.context()?;
    let db = ctx.db.get_cache()?;
    Ok(db.local_review_verdicts().list_by_target(target)?)
}
