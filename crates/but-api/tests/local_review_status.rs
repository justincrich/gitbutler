//! LPR-005 integration proofs (AC-1..AC-4) — real but-db + real gix.
//!
//! Drives the real `but_api::legacy::forge::review_status` verb against a real
//! governed scenario repo with committed `.gitbutler/{permissions,gates}.toml`
//! and a populated but-db cache. The opener principal's DECLARED `kind` in the
//! committed permissions.toml is the source-of-truth for the `agent_authored`
//! tag (read at the target ref, NOT handle-resolution); these tests exercise
//! that derivation plus the derived PR lifecycle (commits + verdict-at-head +
//! open assignments + open threads).
//!
//! No mocks, no insta — real but-db + real gix only.

use anyhow::Context as _;

const FEAT_REF: &str = "refs/heads/feat";

#[tokio::test]
#[serial_test::serial]
async fn review_status_derives_lifecycle_and_agent_tag() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_governed_repo();
    let ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();

    // `agent_opener` is declared `kind = "agent"` in committed permissions.toml.
    // It opens the review and seeds a pending reviewer assignment for `rev`.
    temp_env::async_with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("agent_opener")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        async {
            but_api::legacy::forge::request_review(
                ctx.to_sync(),
                FEAT_REF.to_owned(),
                Some("rev".to_owned()),
            )
            .await
        },
    )
    .await
    .context("request_review as agent_opener must succeed")?;

    let status = temp_env::async_with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("rev")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        async { but_api::legacy::forge::review_status(ctx.to_sync(), FEAT_REF.to_owned()).await },
    )
    .await
    .context("review_status must succeed on a branch with a pending assignment")?;

    assert_eq!(
        status.target, FEAT_REF,
        "review_status target is the queried branch ref"
    );
    assert_eq!(
        status.lifecycle, "AwaitingReview",
        "a branch with an assignment and no verdict-at-head is AwaitingReview"
    );
    assert!(
        !status.assignments.is_empty(),
        "review_status must surface the seeded assignment"
    );
    assert_eq!(
        status
            .assignments
            .iter()
            .filter(|a| a.reviewer_principal == "rev")
            .count(),
        1,
        "the seeded pending assignment for `rev` must be in the assignments list"
    );
    assert!(
        status.agent_authored,
        "agent_authored must be true because the opener's committed entry declares kind = \"agent\""
    );
    assert_eq!(
        status.verdict_at_head, None,
        "no verdict has been recorded at head yet"
    );
    assert_eq!(
        status.open_threads, 0,
        "no comment threads exist on this fresh review"
    );
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn review_status_reflects_approval() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_governed_repo();
    let ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();

    // `human_opener` opens the review (default-human — no `kind` declared).
    temp_env::async_with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("human_opener")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        async {
            but_api::legacy::forge::request_review(
                ctx.to_sync(),
                FEAT_REF.to_owned(),
                Some("rev".to_owned()),
            )
            .await
        },
    )
    .await?;
    // `rev` approves the current HEAD.
    temp_env::async_with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("rev")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        async { but_api::legacy::forge::approve_review(ctx.to_sync(), FEAT_REF.to_owned()).await },
    )
    .await?;

    let status = but_api::legacy::forge::review_status(ctx.to_sync(), FEAT_REF.to_owned()).await?;

    assert_eq!(
        status.lifecycle, "Approved",
        "an approval-at-head derives the Approved lifecycle"
    );
    assert_eq!(
        status.verdict_at_head.as_deref(),
        Some("approved"),
        "verdict_at_head must surface the recorded `approved` literal"
    );
    assert!(
        !status.agent_authored,
        "human_opener has no declared kind, so agent_authored must be false"
    );
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn review_status_reflects_changes_requested() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_governed_repo();
    let ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();

    temp_env::async_with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("human_opener")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        async {
            but_api::legacy::forge::request_review(
                ctx.to_sync(),
                FEAT_REF.to_owned(),
                Some("rev".to_owned()),
            )
            .await
        },
    )
    .await?;
    temp_env::async_with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("rev")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        async {
            but_api::legacy::forge::request_changes_review(
                ctx.to_sync(),
                FEAT_REF.to_owned(),
                Some("needs work".to_owned()),
            )
            .await
        },
    )
    .await?;

    let status = but_api::legacy::forge::review_status(ctx.to_sync(), FEAT_REF.to_owned()).await?;

    assert_eq!(
        status.lifecycle, "ChangesRequested",
        "a changes_requested verdict-at-head (or assignment state) derives ChangesRequested"
    );
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn review_status_two_read_convergence() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_governed_repo();
    let ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();

    // Seed the same drive state two orchestrators would see: opener + an
    // assignment + an unresolved comment thread.
    temp_env::async_with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("agent_opener")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        async {
            but_api::legacy::forge::request_review(
                ctx.to_sync(),
                FEAT_REF.to_owned(),
                Some("rev".to_owned()),
            )
            .await
        },
    )
    .await?;
    seed_unresolved_comment(&ctx, FEAT_REF, "rev", "please revisit", "t-open")?;

    // Two orchestrators read concurrently — both must see byte-identical state
    // (the read model is derived from the same committed ref + cache).
    let (first, second) = tokio::join!(
        but_api::legacy::forge::review_status(ctx.to_sync(), FEAT_REF.to_owned()),
        but_api::legacy::forge::review_status(ctx.to_sync(), FEAT_REF.to_owned()),
    );
    let first = first?;
    let second = second?;

    assert_eq!(
        first.target, second.target,
        "two concurrent reads must converge on the target"
    );
    assert_eq!(
        first.lifecycle, second.lifecycle,
        "two concurrent reads must converge on the lifecycle"
    );
    assert_eq!(
        first.verdict_at_head, second.verdict_at_head,
        "two concurrent reads must converge on verdict_at_head"
    );
    assert_eq!(
        first.agent_authored, second.agent_authored,
        "two concurrent reads must converge on agent_authored"
    );
    assert_eq!(
        first.open_threads, second.open_threads,
        "two concurrent reads must converge on the open-thread count"
    );
    assert_eq!(
        first.assignments.len(),
        second.assignments.len(),
        "two concurrent reads must converge on the assignment count"
    );
    assert_eq!(
        first.open_threads, 1,
        "the seeded unresolved comment thread must be counted"
    );
    assert_eq!(
        first.lifecycle, "AwaitingReview",
        "an assigned-but-not-yet-reviewed branch is AwaitingReview"
    );
    assert!(
        first.agent_authored,
        "agent_opener's declared kind=agent must be surfaced by both reads"
    );
    Ok(())
}

// ---- fixture + helpers -----------------------------------------------------

/// A governed repo for LPR-005 proofs. Principals:
///   - `agent_opener` — pull_requests:write + contents:write, **kind = "agent"**
///     (the auto-tag derivation must mark reviews opened by it as agent-authored)
///   - `human_opener` — pull_requests:write + contents:write (no `kind` → default human)
///   - `rev`          — reviews:write + contents:read (the assigned reviewer)
fn lpr_governed_repo() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "agent_opener"
permissions = ["contents:write", "pull_requests:write"]
kind = "agent"

[[principal]]
id = "human_opener"
permissions = ["contents:write", "pull_requests:write"]

[[principal]]
id = "rev"
permissions = ["reviews:write", "contents:read"]
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

fn seed_unresolved_comment(
    ctx: &but_ctx::Context,
    target: &str,
    author: &str,
    body: &str,
    thread_id: &str,
) -> anyhow::Result<()> {
    let mut db = ctx.db.get_cache_mut()?;
    db.local_review_comments_mut()
        .insert(but_db::LocalReviewComment {
            id: uuid::Uuid::new_v4().to_string(),
            target: target.to_owned(),
            author_principal: author.to_owned(),
            body: body.to_owned(),
            file: None,
            line: None,
            thread_id: thread_id.to_owned(),
            resolved: false,
            created_at: chrono::Utc::now().naive_utc(),
        })?;
    Ok(())
}
