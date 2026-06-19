use but_core::RefMetadata as _;
use but_db::{ForgeReview, LocalReviewVerdict};
use gix::refs::FullName;

use crate::utils::{CommandExt as _, Sandbox};

const REVIEW_ID: usize = 1;

#[test]
#[serial_test::serial]
fn merge_gate_auto_merge_denial_is_structured() -> anyhow::Result<()> {
    let env = governed_review_env()?;

    env.but("--format json pr auto-merge 1")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "impl")
        .assert()
        .failure()
        .stdout_eq(snapbox::str![[r#"
"#]])
        .stderr_eq(snapbox::str![[r#"
Error: Failed to set the auto-merge state.

Caused by:
    {"error":{"code":"perm.denied","message":"action requires merge; authorization denied (held permissions: contents:write, pull_requests:write)","remediation_hint":"request a reviewed merge or ask a maintainer to grant merge","unmet":[]}}

"#]]);

    Ok(())
}

fn governed_review_env() -> anyhow::Result<Sandbox> {
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

[[gate]]
branch = "main"
type = "review"
min_approvals = 1
require_distinct_from_author = true
EOF
git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "governance config"
git checkout A
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
            title: "Merge gate fixture".to_owned(),
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
    ctx.db
        .get_cache_mut()?
        .local_review_verdicts_mut()
        .insert(LocalReviewVerdict {
            id: "reviewer-approval".to_owned(),
            target: "refs/heads/A".to_owned(),
            principal_id: "reviewer".to_owned(),
            verdict: "approved".to_owned(),
            head_oid: head.to_string(),
            created_at: fixed_time(1),
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
