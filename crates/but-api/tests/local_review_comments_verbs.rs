//! Integration proofs for the LPR-004 comment-thread verbs.
//!
//! Each test drives the real `but-api` verbs against a real `but-db` cache
//! backed by a real `gix` fixture via `but_testsupport`. Mirrors the
//! hand-assertion style of `forge_guard.rs` / `merge_gate.rs` — no insta
//! snapshots, no mocks.

use anyhow::Context as _;

const FEAT_REF: &str = "refs/heads/feat";
const MAIN_REF: &str = "refs/heads/main";
const REVIEW_ID: usize = 1;

#[tokio::test]
#[serial_test::serial]
async fn post_comment_persists_comment_with_resolved_false() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_governed_repo();
    let ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();

    let verdicts_before = verdict_count(&ctx)?;
    let assignments_before = assignment_count(&ctx)?;

    temp_env::async_with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("rev")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        async {
            but_api::legacy::forge::post_comment(
                ctx.to_sync(),
                "feat".to_owned(),
                "fix this".to_owned(),
                Some("f.rs".to_owned()),
                Some(12),
                "t1".to_owned(),
            )
            .await
        },
    )
    .await?;

    let (target, author, body, file, line, thread_id, resolved) = {
        let db = ctx.db.get_cache()?;
        let comments = db.local_review_comments().list_by_target("feat")?;
        assert_eq!(
            comments.len(),
            1,
            "post_comment must persist exactly one comment"
        );
        let comment = comments
            .first()
            .context("post_comment wrote a row the test should be able to read")?;
        (
            comment.target.clone(),
            comment.author_principal.clone(),
            comment.body.clone(),
            comment.file.clone(),
            comment.line,
            comment.thread_id.clone(),
            comment.resolved,
        )
    };
    assert_eq!(target, "feat");
    assert_eq!(author, "rev");
    assert_eq!(body, "fix this");
    assert_eq!(
        file.as_deref(),
        Some("f.rs"),
        "file anchors must round-trip"
    );
    assert_eq!(line, Some(12), "line anchors must round-trip");
    assert_eq!(thread_id, "t1");
    assert!(
        !resolved,
        "new comments must be persisted with resolved=false"
    );

    // File=None + line=None round-trip for a PR-level comment.
    {
        let mut db = ctx.db.get_cache_mut()?;
        db.local_review_comments_mut().insert(local_comment_row(
            "feat",
            "rev",
            "branch-level",
            None,
            None,
            "pr-level",
            false,
        ))?;
    }
    let pr_level_file_line = {
        let db = ctx.db.get_cache()?;
        let pr_level = db
            .local_review_comments()
            .list_by_thread("feat", "pr-level")?;
        let pr_level = pr_level
            .first()
            .context("PR-level comment insert must round-trip")?;
        (pr_level.file.clone(), pr_level.line)
    };
    assert!(pr_level_file_line.0.is_none() && pr_level_file_line.1.is_none());

    // No verdicts or assignments may be touched by a comment write.
    assert_eq!(
        verdict_count(&ctx)?,
        verdicts_before,
        "post_comment must not write local_review_verdicts"
    );
    assert_eq!(
        assignment_count(&ctx)?,
        assignments_before,
        "post_comment must not write local_review_assignments"
    );
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn list_comments_returns_threads_excludes_pr_meta() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_governed_repo();
    let ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();

    seed_comment(&ctx, "feat", "rev", "first", Some("f.rs"), Some(1), "t1")?;
    seed_comment(&ctx, "feat", "rev", "second", Some("f.rs"), Some(2), "t1")?;
    seed_comment(&ctx, "feat", "rev", "third", None, None, "t2")?;
    // Seed the reserved __pr_meta__ marker directly via the Handle (the opener lives
    // in local_review_meta — request_review does NOT write a __pr_meta__ comment row).
    seed_comment(
        &ctx,
        "feat",
        "rev",
        "reserved marker",
        None,
        None,
        but_api::legacy::forge::RESERVED_PR_META_THREAD,
    )?;

    let rows = temp_env::async_with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("rev")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        async { but_api::legacy::forge::list_comments(ctx.to_sync(), "feat".to_owned()).await },
    )
    .await?;

    let t1_count = rows.iter().filter(|c| c.thread_id == "t1").count();
    let t2_count = rows.iter().filter(|c| c.thread_id == "t2").count();
    assert_eq!(t1_count, 2, "t1 thread must surface both comments");
    assert_eq!(t2_count, 1, "t2 thread must surface its single comment");
    assert!(
        rows.iter()
            .all(|c| c.thread_id != but_api::legacy::forge::RESERVED_PR_META_THREAD),
        "the reserved __pr_meta__ marker thread must never be surfaced by list_comments"
    );
    assert_eq!(
        rows.len(),
        3,
        "only the three real comments survive the __pr_meta__ filter"
    );
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn resolve_thread_flips_whole_thread() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_governed_repo();
    let ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();

    // Two comments on t1 authored by `rev` (the thread author → a permitted resolver).
    seed_comment(&ctx, "feat", "rev", "first", Some("f.rs"), Some(1), "t1")?;
    seed_comment(&ctx, "feat", "rev", "second", Some("f.rs"), Some(2), "t1")?;
    // Pre-existing verdict and assignment rows — must be byte-unchanged by the resolve.
    seed_verdict(&ctx, "feat", "rev")?;
    seed_assignment(&ctx, "feat", "rev2")?;

    let verdicts_before = verdict_count(&ctx)?;
    let assignments_before = assignment_count(&ctx)?;
    let verdicts_snapshot = snapshot_verdicts(&ctx)?;
    let assignments_snapshot = snapshot_assignments(&ctx)?;

    temp_env::async_with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("rev")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        async {
            but_api::legacy::forge::resolve_thread(
                ctx.to_sync(),
                "feat".to_owned(),
                "t1".to_owned(),
                true,
            )
            .await
        },
    )
    .await?;

    let db = ctx.db.get_cache()?;
    let t1_comments = db.local_review_comments().list_by_thread("feat", "t1")?;
    assert_eq!(
        t1_comments.len(),
        2,
        "resolve_thread must not change the thread's row count"
    );
    assert!(
        t1_comments.iter().all(|c| c.resolved),
        "every comment in the thread must carry resolved=true after a permitted resolve"
    );

    // Verdicts + assignments byte-unchanged.
    assert_eq!(
        verdict_count(&ctx)?,
        verdicts_before,
        "resolve_thread must not touch local_review_verdicts"
    );
    assert_eq!(
        assignment_count(&ctx)?,
        assignments_before,
        "resolve_thread must not touch local_review_assignments"
    );
    assert_eq!(
        snapshot_verdicts(&ctx)?,
        verdicts_snapshot,
        "verdict rows must be byte-identical before/after resolve_thread"
    );
    assert_eq!(
        snapshot_assignments(&ctx)?,
        assignments_snapshot,
        "assignment rows must be byte-identical before/after resolve_thread"
    );
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn post_comment_denied_without_authority_writes_nothing() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_governed_repo();
    let ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();

    let comments_before = comment_count(&ctx, "feat")?;

    let err = temp_env::async_with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("impl")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        async {
            but_api::legacy::forge::post_comment(
                ctx.to_sync(),
                "feat".to_owned(),
                "x".to_owned(),
                None,
                None,
                "t1".to_owned(),
            )
            .await
        },
    )
    .await
    .expect_err("a principal lacking comments:write must not post a comment");

    let gate_error = but_api::legacy::forge::classify_error(&err)
        .context("perm.denied must surface as a structured forge gate error")?;
    assert_eq!(
        gate_error.code, "perm.denied",
        "missing comments:write must be reported as perm.denied"
    );
    assert!(
        gate_error.message.contains("comments:write"),
        "denial message must name comments:write, got: {}",
        gate_error.message
    );

    assert_eq!(
        comment_count(&ctx, "feat")?,
        comments_before,
        "a denied post_comment must write no row (authorize-before-write)"
    );
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn post_comment_is_local_cache_only() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_governed_repo();
    let ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();

    let feat_before = ref_id(&repo, FEAT_REF)?;
    let main_before = ref_id(&repo, MAIN_REF)?;
    let objects_before = object_count(&repo)?;
    let refs_before = ref_count(&repo)?;

    temp_env::async_with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("rev")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        async {
            but_api::legacy::forge::post_comment(
                ctx.to_sync(),
                "feat".to_owned(),
                "x".to_owned(),
                None,
                None,
                "t1".to_owned(),
            )
            .await
        },
    )
    .await?;

    // The row IS written (local-cache only, like approve_review — no DryRun guard).
    assert_eq!(
        comment_count(&ctx, "feat")?,
        1,
        "post_comment must persist the comment to the local cache"
    );

    // ... and refs / objects / oplog are byte-identical.
    assert_eq!(
        ref_id(&repo, FEAT_REF)?,
        feat_before,
        "post_comment must not move the feat ref"
    );
    assert_eq!(
        ref_id(&repo, MAIN_REF)?,
        main_before,
        "post_comment must not move the main ref"
    );
    assert_eq!(
        object_count(&repo)?,
        objects_before,
        "post_comment must not add or remove any git object"
    );
    assert_eq!(
        ref_count(&repo)?,
        refs_before,
        "post_comment must not add or remove any ref"
    );
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn unresolved_comment_does_not_block_merge() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_merge_gated_repo()?;
    let head = ref_id(&repo, FEAT_REF)?;
    let ctx = context_with_review(&repo, head)?;

    // Verdict-at-head: `reviewer` approves the feat HEAD (satisfies min_approvals=1).
    approve_branch(&ctx, "reviewer").await?;

    // An UNRESOLVED comment thread on the same branch — list_comments surfaces it.
    seed_comment(
        &ctx,
        "feat",
        "reviewer",
        "please revisit",
        None,
        None,
        "t-open",
    )?;
    let rows = temp_env::async_with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("reviewer")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        async { but_api::legacy::forge::list_comments(ctx.to_sync(), "feat".to_owned()).await },
    )
    .await?;
    assert!(
        rows.iter().any(|c| c.thread_id == "t-open" && !c.resolved),
        "list_comments must surface the unresolved thread as a drive signal"
    );

    // The governed merge proceeds: the open thread is a drive signal that never
    // reaches the gate. The call fails OUTSIDE governance (network-backed forge
    // call), proving the gate itself passed.
    let err = temp_env::async_with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("maint")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        async { but_api::legacy::forge::merge_review(ctx.to_sync(), REVIEW_ID, None).await },
    )
    .await
    .expect_err("local fixture reaches the forge call and fails outside governance");
    assert!(
        but_api::legacy::forge::classify_error(&err).is_none(),
        "an unresolved comment thread must not gate the merge; the gate must pass at verdict-at-head"
    );
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn unauthorized_resolve_rejected_and_pr_meta_guarded() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_governed_repo();
    let ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();

    // Thread t1 authored by `rev` (the thread author). `rev2` is the assigned reviewer.
    seed_comment(&ctx, "feat", "rev", "first", Some("f.rs"), Some(1), "t1")?;
    seed_comment(&ctx, "feat", "rev", "second", Some("f.rs"), Some(2), "t1")?;
    seed_assignment(&ctx, "feat", "rev2")?;

    // `other` holds comments:write but is NOT the thread author, NOT the assigned
    // reviewer, and NOT a reviews:write holder → must be REJECTED (R22).
    let err = temp_env::async_with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("other")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        async {
            but_api::legacy::forge::resolve_thread(
                ctx.to_sync(),
                "feat".to_owned(),
                "t1".to_owned(),
                true,
            )
            .await
        },
    )
    .await
    .expect_err("a third unrelated principal must not self-resolve another party's thread");
    assert!(
        err.to_string().contains("thread author")
            || err.to_string().contains("assigned reviewer")
            || err.to_string().contains("reviews:write"),
        "R22 denial must explain the resolver-identity constraint, got: {err}"
    );

    {
        let db = ctx.db.get_cache()?;
        let t1_after = db.local_review_comments().list_by_thread("feat", "t1")?;
        assert!(
            t1_after.iter().all(|c| !c.resolved),
            "the rejected resolve must flip no comment row"
        );
    }

    // The thread author (`rev`) CAN resolve.
    temp_env::async_with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("rev")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        async {
            but_api::legacy::forge::resolve_thread(
                ctx.to_sync(),
                "feat".to_owned(),
                "t1".to_owned(),
                true,
            )
            .await
        },
    )
    .await?;
    {
        let db = ctx.db.get_cache()?;
        let t1 = db.local_review_comments().list_by_thread("feat", "t1")?;
        assert!(
            t1.iter().all(|c| c.resolved),
            "the thread author must be permitted to resolve"
        );
    }

    // The assigned reviewer (`rev2`) CAN resolve after we reset the thread.
    set_thread_resolved(&ctx, "feat", "t1", false)?;
    temp_env::async_with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("rev2")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        async {
            but_api::legacy::forge::resolve_thread(
                ctx.to_sync(),
                "feat".to_owned(),
                "t1".to_owned(),
                true,
            )
            .await
        },
    )
    .await?;
    {
        let db = ctx.db.get_cache()?;
        let t1 = db.local_review_comments().list_by_thread("feat", "t1")?;
        assert!(
            t1.iter().all(|c| c.resolved),
            "the assigned reviewer must be permitted to resolve"
        );
    }

    // A reviews:write holder (`rev3`) who is NOT the author and NOT the assigned
    // reviewer CAN resolve after we reset.
    set_thread_resolved(&ctx, "feat", "t1", false)?;
    temp_env::async_with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("rev3")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        async {
            but_api::legacy::forge::resolve_thread(
                ctx.to_sync(),
                "feat".to_owned(),
                "t1".to_owned(),
                true,
            )
            .await
        },
    )
    .await?;
    {
        let db = ctx.db.get_cache()?;
        let t1 = db.local_review_comments().list_by_thread("feat", "t1")?;
        assert!(
            t1.iter().all(|c| c.resolved),
            "a reviews:write holder must be permitted to resolve even when not author/assigned"
        );
    }

    // R23: a caller cannot post to the reserved __pr_meta__ thread.
    let err = temp_env::async_with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("rev")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        async {
            but_api::legacy::forge::post_comment(
                ctx.to_sync(),
                "feat".to_owned(),
                "sentinel".to_owned(),
                None,
                None,
                but_api::legacy::forge::RESERVED_PR_META_THREAD.to_owned(),
            )
            .await
        },
    )
    .await
    .expect_err("the reserved __pr_meta__ thread must be rejected");
    assert!(
        err.to_string().contains("__pr_meta__") && err.to_string().contains("reserved"),
        "post_comment must explain __pr_meta__ is reserved, got: {err}"
    );

    // R23: resolve_thread also refuses the reserved thread.
    let err = temp_env::async_with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("rev")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        async {
            but_api::legacy::forge::resolve_thread(
                ctx.to_sync(),
                "feat".to_owned(),
                but_api::legacy::forge::RESERVED_PR_META_THREAD.to_owned(),
                true,
            )
            .await
        },
    )
    .await
    .expect_err("resolve_thread must refuse the reserved __pr_meta__ thread");
    assert!(
        err.to_string().contains("__pr_meta__") && err.to_string().contains("reserved"),
        "resolve_thread must explain __pr_meta__ is reserved, got: {err}"
    );
    Ok(())
}

// ---------- helpers ----------

fn lpr_governed_repo() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
git remote add origin https://github.com/gitbutler/lpr-comment-fixture.git
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'

[[principal]]
id = "rev"
permissions = ["reviews:write", "comments:write", "contents:read"]

[[principal]]
id = "rev2"
permissions = ["comments:write", "contents:read"]

[[principal]]
id = "rev3"
permissions = ["reviews:write", "comments:write", "contents:read"]

[[principal]]
id = "other"
permissions = ["comments:write", "contents:read"]

[[principal]]
id = "impl"
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

/// A merge-gated repo (min_approvals=1, distinct-from-author) used by
/// `unresolved_comment_does_not_block_merge`. The principal set mirrors
/// `merge_gate.rs`'s Single config so the governed merge is satisfiable.
///
/// The feat branch carries an empty gates.toml so the strict but-authz
/// GatesWire parser (used by `authorize_branch_action`) accepts it; the
/// [[gate]] block lives on main where the merge-gate's permissive parser
/// reads it.
fn lpr_merge_gated_repo() -> anyhow::Result<(gix::Repository, tempfile::TempDir)> {
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
permissions = ["reviews:write", "comments:write"]

[[principal]]
id = "maint"
permissions = ["merge", "reviews:write"]
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
# feat's gates.toml is empty: the but-authz GatesWire parser (used by
# authorize_branch_action on feat) only accepts [[branch]], and the gate
# requirement itself is read from main by the permissive but-api parser.
cat >.gitbutler/gates.toml <<'EOF'
EOF
echo feat >feat.txt
git add .gitbutler/gates.toml feat.txt
git commit -m "feat"
git checkout main
"#,
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
        .upsert(but_db::ForgeReview {
            html_url: "https://github.com/gitbutler/merge-gate-fixture/pull/1".to_owned(),
            number: REVIEW_ID.try_into()?,
            title: "Unresolved comment fixture".to_owned(),
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

fn object_count(repo: &gix::Repository) -> anyhow::Result<usize> {
    let mut count = 0usize;
    let mut iter = repo.objects.iter()?;
    while let Some(item) = iter.next().transpose()? {
        // Only count loose+packed object entries (the iter yields one per object id).
        let _ = item;
        count += 1;
    }
    Ok(count)
}

fn ref_count(repo: &gix::Repository) -> anyhow::Result<usize> {
    Ok(repo.references()?.all()?.filter_map(Result::ok).count())
}

fn local_comment_row(
    target: &str,
    author: &str,
    body: &str,
    file: Option<&str>,
    line: Option<i64>,
    thread_id: &str,
    resolved: bool,
) -> but_db::LocalReviewComment {
    but_db::LocalReviewComment {
        id: uuid::Uuid::new_v4().to_string(),
        target: target.to_owned(),
        author_principal: author.to_owned(),
        body: body.to_owned(),
        file: file.map(str::to_owned),
        line,
        thread_id: thread_id.to_owned(),
        resolved,
        created_at: chrono::Utc::now().naive_utc(),
    }
}

fn seed_comment(
    ctx: &but_ctx::Context,
    target: &str,
    author: &str,
    body: &str,
    file: Option<&str>,
    line: Option<i64>,
    thread_id: &str,
) -> anyhow::Result<()> {
    let mut db = ctx.db.get_cache_mut()?;
    db.local_review_comments_mut().insert(local_comment_row(
        target, author, body, file, line, thread_id, false,
    ))?;
    Ok(())
}

fn seed_verdict(ctx: &but_ctx::Context, target: &str, principal: &str) -> anyhow::Result<()> {
    let mut db = ctx.db.get_cache_mut()?;
    db.local_review_verdicts_mut()
        .insert(but_db::LocalReviewVerdict {
            id: uuid::Uuid::new_v4().to_string(),
            target: target.to_owned(),
            principal_id: principal.to_owned(),
            verdict: "approved".to_owned(),
            head_oid: "0".to_owned(),
            created_at: chrono::Utc::now().naive_utc(),
        })?;
    Ok(())
}

fn seed_assignment(ctx: &but_ctx::Context, target: &str, reviewer: &str) -> anyhow::Result<()> {
    let mut db = ctx.db.get_cache_mut()?;
    db.local_review_assignments_mut()
        .upsert(but_db::LocalReviewAssignment {
            id: uuid::Uuid::new_v4().to_string(),
            target: target.to_owned(),
            reviewer_principal: reviewer.to_owned(),
            state: "assigned".to_owned(),
            assigned_at: chrono::Utc::now().naive_utc(),
        })?;
    Ok(())
}

fn set_thread_resolved(
    ctx: &but_ctx::Context,
    _target: &str,
    thread_id: &str,
    resolved: bool,
) -> anyhow::Result<()> {
    let mut db = ctx.db.get_cache_mut()?;
    db.local_review_comments_mut()
        .set_resolved(thread_id, resolved)?;
    Ok(())
}

fn comment_count(ctx: &but_ctx::Context, target: &str) -> anyhow::Result<usize> {
    Ok(ctx
        .db
        .get_cache()?
        .local_review_comments()
        .list_by_target(target)?
        .len())
}

fn verdict_count(ctx: &but_ctx::Context) -> anyhow::Result<usize> {
    Ok(ctx
        .db
        .get_cache()?
        .local_review_verdicts()
        .list_by_target(FEAT_REF)?
        .len())
}

fn assignment_count(ctx: &but_ctx::Context) -> anyhow::Result<usize> {
    Ok(ctx
        .db
        .get_cache()?
        .local_review_assignments()
        .list_by_target(FEAT_REF)?
        .len())
}

fn snapshot_verdicts(ctx: &but_ctx::Context) -> anyhow::Result<String> {
    let verdicts = ctx
        .db
        .get_cache()?
        .local_review_verdicts()
        .list_by_target(FEAT_REF)?;
    Ok(serde_json::to_string(&verdicts)?)
}

fn snapshot_assignments(ctx: &but_ctx::Context) -> anyhow::Result<String> {
    let assignments = ctx
        .db
        .get_cache()?
        .local_review_assignments()
        .list_by_target(FEAT_REF)?;
    Ok(serde_json::to_string(&assignments)?)
}

async fn approve_branch(ctx: &but_ctx::Context, principal_id: &str) -> anyhow::Result<()> {
    temp_env::async_with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some(principal_id)),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        async { but_api::legacy::forge::approve_review(ctx.to_sync(), "feat".to_owned()).await },
    )
    .await
}
