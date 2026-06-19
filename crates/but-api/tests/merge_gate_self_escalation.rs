use std::{path::Path, str};

use anyhow::Context as _;
use but_api::legacy::merge_gate::{classify_error, enforce_merge_gate};
use but_db::ForgeReview;
use serde::Deserialize;

const MAIN_REF: &str = "refs/heads/main";
const FEAT_REF: &str = "refs/heads/feat";
const REVIEW_ID: usize = 202;

#[tokio::test]
#[serial_test::serial]
async fn self_add_to_maintainers_on_feature_head_still_denied_merge() -> anyhow::Result<()> {
    let (repo, _tmp) = self_escalation_repo();
    let main_before = ref_id(&repo, MAIN_REF)?;
    assert_maintainers_members(&repo, &["maint"], &["feat-author"])?;

    let ctx = context_with_review(&repo, ref_id(&repo, FEAT_REF)?)?;

    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("feat-author"))], async {
        let err = enforce_merge_gate(&ctx, REVIEW_ID)
            .expect_err("feature-head self-add must not clear Authority::Merge");
        let denial = classify_error(&err).expect("merge-authority denial must be structured");
        assert_eq!(
            denial.code, but_authz::Denial::PERM_DENIED_CODE,
            "self-added feature-head membership must still deny merge at the target-ref authority step"
        );
        assert!(
            denial.message.contains("merge"),
            "merge-authority denial must name the missing merge authority"
        );
        Ok::<(), anyhow::Error>(())
    })
    .await?;

    assert_eq!(
        ref_id(&repo, MAIN_REF)?,
        main_before,
        "feature-head self-add must leave refs/heads/main unchanged"
    );

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn landed_membership_clears_merge_authority_step() -> anyhow::Result<()> {
    let (repo, _tmp) = self_escalation_repo();
    let main_before = ref_id(&repo, MAIN_REF)?;
    let ctx = context_with_review(&repo, ref_id(&repo, FEAT_REF)?)?;

    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("feat-author"))], async {
        let err = enforce_merge_gate(&ctx, REVIEW_ID)
            .expect_err("pre-landing self-add must be denied at Authority::Merge");
        let denial = classify_error(&err).expect("pre-landing denial must be structured");
        assert_eq!(
            denial.code,
            but_authz::Denial::PERM_DENIED_CODE,
            "before landing, feat-author must not hold merge on the target ref"
        );
        Ok::<(), anyhow::Error>(())
    })
    .await?;

    land_feat_author_membership(&repo);
    let main_after = ref_id(&repo, MAIN_REF)?;
    assert_ne!(
        main_before, main_after,
        "landing maintainers membership must advance refs/heads/main"
    );
    assert_maintainers_members(&repo, &["maint", "feat-author"], &[])?;

    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("feat-author"))], async {
        let err = enforce_merge_gate(&ctx, REVIEW_ID)
            .expect_err("unapproved review gate should be the expected next gate");
        let denial = classify_error(&err).expect("review gate denial must be structured");
        assert_ne!(
            denial.code, but_authz::Denial::PERM_DENIED_CODE,
            "after landing, Authority::Merge perm.denied must be gone"
        );
        assert_eq!(
            denial.code, "gate.review_required",
            "after merge authority clears, the unapproved review requirement is the expected next gate"
        );
        Ok::<(), anyhow::Error>(())
    })
    .await?;

    Ok(())
}

fn self_escalation_repo() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
git remote add origin https://github.com/gitbutler/merge-gate-fixture.git
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "feat-author"
permissions = ["contents:write"]

[[principal]]
id = "maint"
permissions = ["merge"]

[[group]]
name = "maintainers"
permissions = ["merge"]
members = ["maint"]
EOF
cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true

[[gate]]
branch = "main"
type = "review"
min_approvals = 1
EOF
git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "target governance excludes feat author"
git checkout -b feat
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "feat-author"
permissions = ["contents:write"]

[[principal]]
id = "maint"
permissions = ["merge"]

[[group]]
name = "maintainers"
permissions = ["merge"]
members = ["maint", "feat-author"]
EOF
echo feature >feat.txt
git add .gitbutler/permissions.toml feat.txt
git commit -m "self add feat author to maintainers"
git checkout main
"#,
        &repo,
    );
    (repo, tmp)
}

fn land_feat_author_membership(repo: &gix::Repository) {
    but_testsupport::invoke_bash(
        r#"
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "feat-author"
permissions = ["contents:write"]

[[principal]]
id = "maint"
permissions = ["merge"]

[[group]]
name = "maintainers"
permissions = ["merge"]
members = ["maint", "feat-author"]
EOF
git add .gitbutler/permissions.toml
git commit -m "land feat author maintainers membership"
"#,
        repo,
    );
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
            html_url: "https://github.com/gitbutler/merge-gate-fixture/pull/202".to_owned(),
            number: REVIEW_ID.try_into()?,
            title: "Self escalation fixture".to_owned(),
            body: None,
            author: Some("feat-author".to_owned()),
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

fn assert_maintainers_members(
    repo: &gix::Repository,
    expected_present: &[&str],
    expected_absent: &[&str],
) -> anyhow::Result<()> {
    let permissions = target_ref_permissions(repo)?;
    let maintainers = permissions
        .group
        .iter()
        .find(|group| group.name == "maintainers")
        .ok_or_else(|| anyhow::anyhow!("target-ref maintainers group must exist"))?;

    for member in expected_present {
        assert!(
            maintainers
                .members
                .iter()
                .any(|candidate| candidate == member),
            "target-ref maintainers must include {member}"
        );
    }

    for member in expected_absent {
        assert!(
            !maintainers
                .members
                .iter()
                .any(|candidate| candidate == member),
            "target-ref maintainers must exclude {member}"
        );
    }

    Ok(())
}

fn target_ref_permissions(repo: &gix::Repository) -> anyhow::Result<PermissionsWire> {
    let mut reference = repo
        .find_reference(MAIN_REF)
        .with_context(|| format!("resolving target ref {MAIN_REF}"))?;
    let commit = reference
        .peel_to_commit()
        .with_context(|| format!("peeling {MAIN_REF} to a commit"))?;
    let tree = commit
        .tree()
        .with_context(|| format!("reading tree for {MAIN_REF}"))?;
    let entry = tree
        .lookup_entry_by_path(Path::new(".gitbutler/permissions.toml"))
        .with_context(|| format!("looking up permissions.toml in {MAIN_REF}"))?
        .ok_or_else(|| anyhow::anyhow!("missing permissions.toml at {MAIN_REF}"))?;
    let blob = repo
        .find_blob(entry.id())
        .with_context(|| format!("reading permissions.toml blob at {MAIN_REF}"))?;
    let content = str::from_utf8(&blob.data)
        .with_context(|| format!("decoding permissions.toml at {MAIN_REF} as UTF-8"))?;

    Ok(toml::from_str(content)?)
}

#[derive(Debug, Deserialize)]
struct PermissionsWire {
    #[serde(default)]
    group: Vec<GroupWire>,
}

#[derive(Debug, Deserialize)]
struct GroupWire {
    name: String,
    #[serde(default)]
    members: Vec<String>,
}
