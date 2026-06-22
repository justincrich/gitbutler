//! LPR-007 AC-5 — the auto-opened assignment is visible in `but review status`,
//! closing the commit->dispatch loop WITHOUT an explicit `but review request`.
//!
//! Drives the real `but_api::legacy::forge::review_status` verb against a real
//! governed scenario repo. The LPR-007 hook (`but_rules::process_commit_rules`)
//! is fired DIRECTLY against the cache handle — bypassing the explicit
//! `but review request` writer — to prove the auto-opened pending assignment
//! surfaces in `review_status` so the reconciler dispatches the reviewer.
//!
//! No mocks, no insta — real but-db + real gix only.

use anyhow::Context as _;

const FEAT_REF: &str = "refs/heads/feat";

/// AC-5: the auto-opened pending assignment appears in `but review status`.
/// The LPR-007 hook fires on commit (simulated here by calling
/// `but_rules::process_commit_rules` directly with a fixtured `CommitContext`
/// that matches the rule's filter) and opens a `pending` drive-only
/// `local_review_assignments` row. `review_status` must surface that row
/// without an explicit `but_api::legacy::forge::request_review` call, closing
/// the commit->reviewer-dispatch loop.
#[tokio::test]
#[serial_test::serial]
async fn auto_assignment_visible_in_review_status() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_governed_repo();
    let ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();

    // Sanity: the fixtured branch ref exists so review_status can resolve HEAD.
    let _head = ref_id(&ctx, FEAT_REF)?;

    // Fire the LPR-007 hook directly via the but-rules engine. This opens a
    // pending `local_review_assignments` row for `rev` WITHOUT going through
    // `but_api::legacy::forge::request_review` — the auto-open path the
    // reconciler must observe. A rule filter scoped to principal `dev` is
    // satisfied by the fixtured CommitContext below.
    {
        let mut db = ctx.db.get_cache_mut()?;
        let rule = but_rules::WorkspaceRule::for_test(
            "lpr-007-auto-open",
            but_rules::Trigger::Commit,
            vec![but_rules::Filter::ClaudeCodeSessionId("dev".to_owned())],
            but_rules::Action::RequestReview(but_rules::RequestReviewAction {
                reviewer: "rev".to_owned(),
            }),
        );
        let commit_ctx = but_rules::CommitContext {
            target: FEAT_REF.to_owned(),
            session_id: Some("dev".to_owned()),
            changed_paths: vec![],
        };
        let written = but_rules::process_commit_rules(vec![rule], &commit_ctx, &mut db)
            .context("the LPR-007 hook must fire without error")?;
        assert_eq!(
            written, 1,
            "the hook must write exactly one pending assignment for `rev` on feat"
        );
    }

    // Now read review_status. The auto-opened assignment MUST appear — no
    // explicit `but review request` was issued.
    let status = but_api::legacy::forge::review_status(ctx.to_sync(), FEAT_REF.to_owned())
        .await
        .context("review_status must succeed with the auto-opened assignment present")?;

    assert_eq!(
        status.target, FEAT_REF,
        "review_status target echoes the queried branch ref"
    );
    assert!(
        status
            .assignments
            .iter()
            .any(|a| a.reviewer_principal == "rev" && a.state == "pending"),
        "the auto-opened pending assignment for `rev` must surface in review_status.assignments — got {:?}",
        status.assignments
    );
    assert!(
        status
            .open_assignments
            .iter()
            .any(|a| a.reviewer_principal == "rev" && a.state == "pending"),
        "the auto-opened pending assignment for `rev` must also surface in review_status.open_assignments (the reconciler dispatch trigger) — got {:?}",
        status.open_assignments
    );

    Ok(())
}

// ----fixture helpers --------------------------------------------------------

/// A governed repo for the LPR-007 AC-5 proof. Principals:
/// - `dev`: pull_requests:write + contents:write (the committer whose
///   commit matches the rule's ClaudeCodeSessionId filter)
/// - `rev`: reviews:write + contents:read (the auto-assigned reviewer)
fn lpr_governed_repo() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "dev"
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

fn ref_id(ctx: &but_ctx::Context, ref_name: &str) -> anyhow::Result<gix::ObjectId> {
    let repo = ctx.repo.get()?;
    Ok(repo.find_reference(ref_name)?.peel_to_id()?.detach())
}
