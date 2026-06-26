use but_core::RefMetadata as _;
use but_db::{ForgeReview, LocalReviewVerdict};
use gix::refs::FullName;

use crate::utils::{CommandExt as _, Sandbox};

const REVIEW_ID: usize = 1;

#[test]
#[serial_test::serial]
fn merge_denial_is_structured_for_implementer_without_merge_authority() -> anyhow::Result<()> {
    let env = governed_review_env()?;

    env.but("--format json pr merge 1 --method squash")
        .allow_json()
        .env("BUT_AUTHZ_ALLOW_ENV_HANDLE", "1").env("BUT_AGENT_HANDLE", "impl")
        .assert()
        .failure()
        .stdout_eq(snapbox::str![[r#"
"#]])
        .stderr_eq(snapbox::str![[r#"
Error: Failed to merge review.

Caused by:
    {"error":{"code":"perm.denied","message":"action requires merge; authorization denied (held permissions: contents:write, pull_requests:write)","remediation_hint":"request a reviewed merge or ask a maintainer to grant merge","class":"actor_correctable","held_permissions":["contents:write","pull_requests:write"],"authorized_actions":[{"command":"but review new","effect":"open a local review for a branch to hand off for review"},{"command":"but perm list","effect":"list your own effective permissions (self-discovery)"}],"unmet":[]}}

"#]]);

    Ok(())
}

#[test]
#[serial_test::serial]
fn merge_dry_run_denial_leaves_refs_and_cache_unchanged() -> anyhow::Result<()> {
    let env = governed_review_env()?;
    let repo = env.open_repo()?;
    let main_before = ref_id(&repo, "refs/heads/main")?;
    let branch_before = ref_id(&repo, "refs/heads/A")?;
    let review_count_before = review_count(&env)?;
    let verdict_count_before = verdict_count(&env)?;

    let output = env
        .but("--format json pr merge 1 --dry-run")
        .allow_json()
        .env("BUT_AUTHZ_ALLOW_ENV_HANDLE", "1")
        .env("BUT_AGENT_HANDLE", "impl")
        .output()?;
    assert!(
        !output.status.success(),
        "implementer dry-run merge without merge authority must fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(r#""code":"perm.denied""#),
        "dry-run denial must be structured as perm.denied, got: {stderr}"
    );
    assert!(
        stderr.contains("action requires merge"),
        "dry-run denial must name the missing merge authority, got: {stderr}"
    );
    assert!(
        !stderr.contains("unrecognized subcommand"),
        "dry-run denial must prove the merge subcommand parsed, got: {stderr}"
    );

    assert_eq!(
        ref_id(&repo, "refs/heads/main")?,
        main_before,
        "denied dry-run merge must leave the target ref unchanged"
    );
    assert_eq!(
        ref_id(&repo, "refs/heads/A")?,
        branch_before,
        "denied dry-run merge must leave the review source ref unchanged"
    );
    assert_eq!(
        review_count(&env)?,
        review_count_before,
        "denied dry-run merge must not rewrite cached review rows"
    );
    assert_eq!(
        verdict_count(&env)?,
        verdict_count_before,
        "denied dry-run merge must not write local review verdict rows"
    );

    Ok(())
}

#[test]
#[serial_test::serial]
fn merge_dry_run_fails_closed_without_agent_handle() -> anyhow::Result<()> {
    let env = governed_review_env()?;

    for (label, handle) in [("unset", None), ("empty", Some(""))] {
        let mut cmd = env.but("--format json pr merge 1 --dry-run").allow_json();
        cmd = match handle {
            Some(value) => cmd
                .env("BUT_AUTHZ_ALLOW_ENV_HANDLE", "1")
                .env("BUT_AGENT_HANDLE", value),
            None => cmd
                .env("BUT_AUTHZ_ALLOW_ENV_HANDLE", "1")
                .env_remove("BUT_AGENT_HANDLE"),
        };

        let output = cmd.output()?;
        assert!(
            !output.status.success(),
            "{label} BUT_AGENT_HANDLE must fail closed"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(r#""code":"perm.denied""#),
            "{label} handle denial must be structured as perm.denied, got: {stderr}"
        );
        assert!(
            stderr.contains("BUT_AGENT_HANDLE is required"),
            "{label} denial must name the missing governed principal handle, got: {stderr}"
        );
        assert!(
            !stderr.contains("unrecognized subcommand"),
            "{label} handle denial must prove the merge subcommand parsed, got: {stderr}"
        );
    }

    Ok(())
}

#[test]
#[serial_test::serial]
fn non_dry_run_reaches_forge_merge_boundary_after_gate_passes() -> anyhow::Result<()> {
    let env = governed_review_env()?;
    let repo = env.open_repo()?;
    let main_before = ref_id(&repo, "refs/heads/main")?;

    let output = env
        .but("--format json pr merge 1 --method rebase")
        .allow_json()
        .env("BUT_AUTHZ_ALLOW_ENV_HANDLE", "1")
        .env("BUT_AGENT_HANDLE", "maint")
        .output()?;

    assert!(
        !output.status.success(),
        "local fixture has no forge credentials, so the real forge merge boundary should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("forge merge_review boundary rejected review 1"),
        "authorized non-dry-run merge must reach the forge merge_review boundary, got: {stderr}"
    );
    assert!(
        !stderr.contains(r#""code":"perm.denied""#),
        "maint has merge authority and current approval, so failure must not be a gate denial: {stderr}"
    );
    assert!(
        !stderr.contains("unrecognized subcommand"),
        "authorized non-dry-run merge must prove the merge subcommand parsed, got: {stderr}"
    );
    assert!(
        !stderr.contains("Failed to set the auto-merge state"),
        "governed merge must not dispatch to auto-merge: {stderr}"
    );
    assert!(
        !stderr.contains("Merged review"),
        "fixture without forge credentials must not report a successful forge merge: {stderr}"
    );
    assert_eq!(
        ref_id(&repo, "refs/heads/main")?,
        main_before,
        "governed forge merge must not run the local but merge command"
    );

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
            target: "A".to_owned(),
            principal_id: "reviewer".to_owned(),
            verdict: "approved".to_owned(),
            head_oid: head.to_string(),
            created_at: fixed_time(1),
        })?;
    Ok(())
}

fn review_count(env: &Sandbox) -> anyhow::Result<usize> {
    let ctx = env.context()?;
    Ok(ctx.db.get_cache()?.forge_reviews().list_all()?.len())
}

fn verdict_count(env: &Sandbox) -> anyhow::Result<usize> {
    let ctx = env.context()?;
    Ok(ctx
        .db
        .get_cache()?
        .local_review_verdicts()
        .list_by_target("A")?
        .len())
}

fn fixed_time(seconds: i64) -> chrono::NaiveDateTime {
    chrono::DateTime::from_timestamp(1_735_689_600 + seconds, 0)
        .expect("fixed timestamp is valid")
        .naive_utc()
}

fn ref_id(repo: &gix::Repository, ref_name: &str) -> anyhow::Result<gix::ObjectId> {
    Ok(repo.find_reference(ref_name)?.peel_to_id()?.detach())
}
