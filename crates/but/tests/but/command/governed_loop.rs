use but_core::RefMetadata as _;
use but_db::ForgeReview;
use gix::refs::FullName;

use crate::utils::{CommandExt as _, Sandbox};

const REVIEW_ID: usize = 77;

#[test]
#[serial_test::serial]
fn governed_loop_reference_flow_full_loop() -> anyhow::Result<()> {
    let env = governed_loop_env("feat", REVIEW_ID)?;
    let repo = env.open_repo()?;
    let main_before = ref_id(&repo, "refs/heads/main")?;
    let feat_before = ref_id(&repo, "refs/heads/feat")?;

    env.file("feature.txt", "feature work\n");
    env.but("--format json commit feat -m feature-work")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "implementer")
        .assert()
        .success();
    let feat_after_implementer = ref_id(&repo, "refs/heads/feat")?;
    assert_ne!(
        feat_after_implementer, feat_before,
        "implementer has contents:write, so committing to the feature branch must advance it"
    );
    update_cached_review_head(&env, "feat", REVIEW_ID)?;

    let pr_new = env
        .but("--format json pr new feat -m 'Feature work'")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "implementer")
        .output()?;
    let pr_new_stderr = String::from_utf8_lossy(&pr_new.stderr);
    assert!(
        !pr_new_stderr.contains(r#""code":"perm.denied""#),
        "implementer has pull_requests:write, so PR publication must not be a governance denial: {pr_new_stderr}"
    );

    assert_merge_denied_for_implementer(&env)?;
    assert_eq!(
        ref_id(&repo, "refs/heads/main")?,
        main_before,
        "denied implementer merge must leave main unchanged"
    );

    let feat_before_reviewer = ref_id(&repo, "refs/heads/feat")?;
    env.file("reviewer-change.txt", "reviewer change\n");
    let reviewer_commit = env
        .but("--format json commit feat -m reviewer-change")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "reviewer")
        .output()?;
    assert_denial(
        &reviewer_commit,
        r#""code":"perm.denied""#,
        "contents:write",
        "reviewer commit must be denied because reviews:write does not imply contents:write",
    );
    assert_eq!(
        ref_id(&repo, "refs/heads/feat")?,
        feat_before_reviewer,
        "denied reviewer commit must leave feat unchanged"
    );

    let zero_approval_merge = env
        .but("--format json pr merge 77 --method squash")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "maintainer")
        .output()?;
    assert_denial(
        &zero_approval_merge,
        r#""code":"gate.review_required""#,
        "review requirement",
        "maintainer merge with no distinct approval must be denied by the review gate",
    );
    assert_eq!(
        ref_id(&repo, "refs/heads/main")?,
        main_before,
        "zero-approval merge denial must leave main unchanged"
    );

    env.but("--format json review approve feat")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "reviewer")
        .assert()
        .success();

    let maintainer_merge = env
        .but("--format json pr merge 77 --method squash")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "maintainer")
        .output()?;
    assert_forge_boundary_after_gate(&maintainer_merge, REVIEW_ID);
    assert_eq!(
        ref_id(&repo, "refs/heads/main")?,
        main_before,
        "local fixture has no forge completion, so permitted forge merge must not move main locally"
    );

    Ok(())
}

#[test]
#[serial_test::serial]
fn governed_loop_remediation_traversable() -> anyhow::Result<()> {
    let env = governed_loop_env("feat", REVIEW_ID)?;
    let repo = env.open_repo()?;
    let main_before = ref_id(&repo, "refs/heads/main")?;

    let denied_merge = env
        .but("--format json pr merge 77 --method squash")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "implementer")
        .output()?;
    assert_denial(
        &denied_merge,
        r#""code":"perm.denied""#,
        "request a reviewed merge",
        "implementer merge denial must carry a traversable remediation hint",
    );

    env.file("remediated.txt", "remediated feature\n");
    env.but("--format json commit feat -m remediated-feature")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "implementer")
        .assert()
        .success();
    update_cached_review_head(&env, "feat", REVIEW_ID)?;

    let pr_new = env
        .but("--format json pr new feat -m 'Remediated feature'")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "implementer")
        .output()?;
    assert_no_governance_denial(&pr_new, "remediation PR creation");

    env.but("--format json review approve feat")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "reviewer")
        .assert()
        .success();

    let maintainer_merge = env
        .but("--format json pr merge 77 --method squash")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "maintainer")
        .output()?;
    assert_forge_boundary_after_gate(&maintainer_merge, REVIEW_ID);
    assert_eq!(
        ref_id(&repo, "refs/heads/main")?,
        main_before,
        "rescope: permitted local forge-bound merge does not fake remote landing"
    );

    Ok(())
}

#[test]
#[serial_test::serial]
fn governed_loop_dryrun_no_bypass() -> anyhow::Result<()> {
    let env = governed_loop_env("feat", REVIEW_ID)?;
    let repo = env.open_repo()?;
    let main_before = ref_id(&repo, "refs/heads/main")?;
    let feat_before = ref_id(&repo, "refs/heads/feat")?;
    let object_count_before = object_count(&env);

    let dry_run = env
        .but("--format json pr merge 77 --dry-run")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "implementer")
        .output()?;
    assert_denial(
        &dry_run,
        r#""code":"perm.denied""#,
        "merge",
        "dry-run merge by an implementer without merge authority must still be denied",
    );
    assert_eq!(
        ref_id(&repo, "refs/heads/main")?,
        main_before,
        "denied dry-run merge must leave main unchanged"
    );
    assert_eq!(
        ref_id(&repo, "refs/heads/feat")?,
        feat_before,
        "denied dry-run merge must leave the source branch unchanged"
    );
    assert_eq!(
        object_count(&env),
        object_count_before,
        "denied dry-run merge must not persist new git objects"
    );

    Ok(())
}

#[test]
#[serial_test::serial]
fn governed_loop_auto_merge_denied() -> anyhow::Result<()> {
    let env = governed_loop_env("feat", REVIEW_ID)?;
    let repo = env.open_repo()?;
    let main_before = ref_id(&repo, "refs/heads/main")?;

    let auto_merge = env
        .but("--format json pr auto-merge 77")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "implementer")
        .output()?;
    assert_denial(
        &auto_merge,
        r#""code":"perm.denied""#,
        "merge",
        "auto-merge must be gated by the same merge authority as explicit merge",
    );
    assert_eq!(
        ref_id(&repo, "refs/heads/main")?,
        main_before,
        "denied auto-merge must leave main unchanged"
    );

    Ok(())
}

#[test]
#[serial_test::serial]
fn governed_loop_unset_handle_failclosed() -> anyhow::Result<()> {
    let env = governed_loop_env("feat", REVIEW_ID)?;
    let repo = env.open_repo()?;
    let main_before = ref_id(&repo, "refs/heads/main")?;

    for (label, handle) in [("unset", None), ("empty", Some(""))] {
        let mut cmd = env.but("--format json pr merge 77 --dry-run").allow_json();
        cmd = match handle {
            Some(value) => cmd.env("BUT_AGENT_HANDLE", value),
            None => cmd.env_remove("BUT_AGENT_HANDLE"),
        };
        let output = cmd.output()?;
        assert_denial(
            &output,
            r#""code":"perm.denied""#,
            "BUT_AGENT_HANDLE",
            &format!("{label} principal handle must fail closed with a structured denial"),
        );
        assert_eq!(
            ref_id(&repo, "refs/heads/main")?,
            main_before,
            "{label} handle denial must leave main unchanged"
        );
    }

    Ok(())
}

fn governed_loop_env(branch_name: &str, review_id: usize) -> anyhow::Result<Sandbox> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack")?;
    env.invoke_bash(format!(
        r#"
git remote set-url origin https://github.com/gitbutler/governed-loop-fixture.git
git branch -f main origin/main
git branch -m A {branch_name}
write_governance_commit() {{
    target_ref="$1"
    base=$(git rev-parse "$target_ref")
    index=$(mktemp)
    export GIT_INDEX_FILE="$index"
    git read-tree "$base"
    permissions_blob=$(git hash-object -w --stdin <<'EOF'
[[principal]]
id = "implementer"
permissions = ["contents:write", "pull_requests:write"]

[[principal]]
id = "reviewer"
groups = ["code-reviewers"]

[[principal]]
id = "maintainer"
groups = ["maintainers"]

[[group]]
name = "code-reviewers"
permissions = ["reviews:write"]
members = ["reviewer"]

[[group]]
name = "maintainers"
permissions = ["merge"]
members = ["maintainer"]
EOF
)
    gates_main_blob=$(git hash-object -w --stdin <<'EOF'
[[branch]]
name = "main"
protected = true

[[gate]]
branch = "main"
type = "review"
min_approvals = 1
require_distinct_from_author = true
EOF
)
    gates_branch_blob=$(git hash-object -w --stdin <<'EOF'
[[branch]]
name = "main"
protected = true

[[branch]]
name = "{branch_name}"
protected = false
EOF
)
    git update-index --add --cacheinfo 100644 "$permissions_blob" .gitbutler/permissions.toml
    if test "$target_ref" = "refs/heads/main"
    then
        git update-index --add --cacheinfo 100644 "$gates_main_blob" .gitbutler/gates.toml
    else
        git update-index --add --cacheinfo 100644 "$gates_branch_blob" .gitbutler/gates.toml
    fi
    tree=$(git write-tree)
    commit=$(printf 'governance config\n' | git commit-tree "$tree" -p "$base")
    git update-ref "$target_ref" "$commit"
    rm "$index"
    unset GIT_INDEX_FILE
}}
write_governance_commit refs/heads/main
write_governance_commit refs/heads/{branch_name}
git checkout {branch_name}
"#
    ));
    env.but("setup").assert().success();
    env.set_target_sha("refs/heads/main")?;
    env.setup_metadata(&[branch_name])?;
    env.but(format!("apply {branch_name}")).assert().success();
    attach_review_id(&env, branch_name, review_id)?;
    upsert_cached_review(&env, branch_name, review_id)?;
    Ok(env)
}

fn assert_merge_denied_for_implementer(env: &Sandbox) -> anyhow::Result<()> {
    let output = env
        .but("--format json pr merge 77 --method squash")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "implementer")
        .output()?;
    assert_denial(
        &output,
        r#""code":"perm.denied""#,
        "merge",
        "implementer lacks merge authority, so explicit merge must be denied",
    );
    Ok(())
}

fn assert_denial(output: &std::process::Output, code: &str, expected_text: &str, reason: &str) {
    assert!(!output.status.success(), "{reason}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(code),
        "{reason}; expected denial code {code}, got: {stderr}"
    );
    assert!(
        stderr.contains(expected_text),
        "{reason}; expected text {expected_text:?}, got: {stderr}"
    );
}

fn assert_no_governance_denial(output: &std::process::Output, label: &str) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains(r#""code":"perm.denied""#)
            && !stderr.contains(r#""code":"branch.protected""#)
            && !stderr.contains(r#""code":"gate.review_required""#),
        "{label} must not fail with a governance denial: {stderr}"
    );
}

fn assert_forge_boundary_after_gate(output: &std::process::Output, review_id: usize) {
    assert!(
        !output.status.success(),
        "local fixture has no forge credentials, so the permitted merge should fail at the forge boundary"
    );
    assert_no_governance_denial(output, "authorized maintainer merge");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(&format!(
            "forge merge_review boundary rejected review {review_id}"
        )),
        "authorized merge must reach the forge merge_review boundary, got: {stderr}"
    );
}

fn attach_review_id(env: &Sandbox, branch_name: &str, review_id: usize) -> anyhow::Result<()> {
    let mut meta = env.meta()?;
    let ref_name = FullName::try_from(format!("refs/heads/{branch_name}"))?;
    let mut branch = meta.branch(ref_name.as_ref())?;
    branch.review.pull_request = Some(review_id);
    meta.set_branch(&branch)?;
    Ok(())
}

fn update_cached_review_head(
    env: &Sandbox,
    branch_name: &str,
    review_id: usize,
) -> anyhow::Result<()> {
    upsert_cached_review(env, branch_name, review_id)
}

fn upsert_cached_review(env: &Sandbox, branch_name: &str, review_id: usize) -> anyhow::Result<()> {
    let repo = env.open_repo()?;
    let head = ref_id(&repo, &format!("refs/heads/{branch_name}"))?;
    let ctx = env.context()?;
    ctx.db
        .get_cache_mut()?
        .forge_reviews_mut()?
        .upsert(ForgeReview {
            html_url: format!(
                "https://github.com/gitbutler/governed-loop-fixture/pull/{review_id}"
            ),
            number: review_id.try_into()?,
            title: format!("Governed loop {branch_name}"),
            body: None,
            author: Some("implementer".to_owned()),
            labels: "[]".to_owned(),
            draft: false,
            source_branch: branch_name.to_owned(),
            target_branch: "main".to_owned(),
            sha: head.to_string(),
            created_at: None,
            modified_at: None,
            merged_at: None,
            closed_at: None,
            repository_ssh_url: None,
            repository_https_url: Some(
                "https://github.com/gitbutler/governed-loop-fixture.git".to_owned(),
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

fn object_count(env: &Sandbox) -> usize {
    env.invoke_git("rev-list --objects --all").lines().count()
}

fn fixed_time(seconds: i64) -> chrono::NaiveDateTime {
    chrono::DateTime::from_timestamp(1_735_689_600 + seconds, 0)
        .expect("fixed timestamp is valid")
        .naive_utc()
}

fn ref_id(repo: &gix::Repository, ref_name: &str) -> anyhow::Result<gix::ObjectId> {
    Ok(repo.find_reference(ref_name)?.peel_to_id()?.detach())
}
