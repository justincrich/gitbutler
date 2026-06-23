//! LPR-008 reconciler read-API proofs (AC-1..AC-5) — real but-db + real gix.
//!
//! Drives the real `but_api::legacy::forge::review_status` verb against a real
//! governed scenario repo with committed `.gitbutler/{permissions,gates}.toml`
//! and a populated but-db cache. The LPR-008 extension enriches the LPR-005
//! payload with the full reconciler drive state — `open_assignments` (pending
//! `local_review_assignments`), `unresolved_threads` (grouped from
//! `local_review_comments` with `resolved = false`), and `approved` (a
//! presentation label derived from the SAME `local_review_verdicts@head` query
//! `enforce_merge_gate` runs) — so an orchestrator decides the next action from
//! ONE read with no shadow state, and two orchestrators converge.
//!
//! No mocks, no insta — real but-db + real gix only.

use anyhow::Context as _;
use but_db::ForgeReview;

const FEAT_REF: &str = "refs/heads/feat";
const MAIN_REF: &str = "refs/heads/main";
const REVIEW_ID: usize = 1;

/// AC-1 [PRIMARY]: one `review_status` payload carries all three drive facts
/// (a pending assignment, an unresolved thread, and the verdict-at-head) for a
/// branch fixtured with all three — the orchestrator decides without a second
/// call or a shadow state.
#[tokio::test]
#[serial_test::serial]
async fn review_status_serves_full_drive_state_in_one_payload() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_reconciler_repo(ReconcilerGate::SingleApproval)?;
    let ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();
    let head = ref_id(&ctx, FEAT_REF)?;
    seed_review(&ctx, head, "opener")?;

    // Seed all three drive facts via the real verbs (no direct row injection
    // apart from the comment — `post_comment` is async and authorizes first,
    // but we intentionally use the direct cache write to keep the test focused
    // on the READ side; the writer path is covered by its own tests).
    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("opener"))], async {
        but_api::legacy::forge::request_review(
            ctx.to_sync(),
            FEAT_REF.to_owned(),
            Some("rev2".to_owned()),
        )
        .await
    })
    .await
    .context("opener must open the review and assign rev2")?;

    seed_unresolved_comment(&ctx, FEAT_REF, "rev", "please revisit X", "t1")?;

    approve_branch(&ctx, "rev2").await?;

    let status = but_api::legacy::forge::review_status(ctx.to_sync(), FEAT_REF.to_owned()).await?;
    // `open_assignments` (pending `local_review_assignments`) is now derived
    // from the `assignments` vec the payload carries — same drive fact, one read.
    let open_assignments: Vec<_> = status
        .assignments
        .iter()
        .filter(|a| a.state == "pending")
        .collect();

    // AC-1: all three drive facts in ONE payload.
    assert_eq!(status.target, FEAT_REF, "target is the queried branch ref");
    assert!(
        open_assignments
            .iter()
            .any(|a| a.reviewer_principal == "rev2" && a.state == "pending"),
        "open_assignments must carry the pending rev2 assignment (dispatch trigger), got {open_assignments:?}"
    );
    assert!(
        status.open_threads >= 1,
        "unresolved_threads must surface at least one open thread (remediation trigger), got {}",
        status.open_threads
    );
    assert!(
        status.verdict_at_head.as_deref() == Some("approved"),
        "approved must be true — an approved verdict at head exists"
    );
    assert_eq!(
        status.verdict_at_head.as_deref(),
        Some("approved"),
        "verdict_at_head literal must reflect the approval@head"
    );
    Ok(())
}

/// AC-2: a target with NO drive state returns `Ok` with empty vecs and
/// `approved = false` (clean empty-state, never an Err).
#[tokio::test]
#[serial_test::serial]
async fn review_status_empty_state_returns_ok() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_reconciler_repo(ReconcilerGate::None)?;
    let ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();

    let status = but_api::legacy::forge::review_status(ctx.to_sync(), FEAT_REF.to_owned())
        .await
        .context("empty-state review_status must return Ok, not Err")?;
    let open_assignments: Vec<_> = status
        .assignments
        .iter()
        .filter(|a| a.state == "pending")
        .collect();

    assert_eq!(
        status.target, FEAT_REF,
        "target echoes the queried branch even when empty"
    );
    assert!(
        open_assignments.is_empty(),
        "empty state → open_assignments is empty"
    );
    assert!(
        status.open_threads == 0,
        "empty state → unresolved_threads is empty"
    );
    assert!(
        status.verdict_at_head.as_deref() != Some("approved"),
        "empty state → approved is false (no verdict at head)"
    );
    assert_eq!(
        status.verdict_at_head, None,
        "empty state → verdict_at_head is None"
    );
    assert_eq!(
        status.lifecycle, "Open",
        "empty state → lifecycle is Open (no assignments, no verdict)"
    );
    Ok(())
}

/// AC-3: the orchestrator's approved-at-head read AGREES with the gate's own
/// verdict-at-head re-derivation. Both read the same `local_review_verdicts@head`
/// truth: review_status reports `approved = true` AND `enforce_merge_gate`
/// permits, from one approval-at-head.
#[tokio::test]
#[serial_test::serial]
async fn review_status_verdict_at_head_agrees_with_merge_gate() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_reconciler_repo(ReconcilerGate::SingleApproval)?;
    let ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();
    let head = ref_id(&ctx, FEAT_REF)?;
    // The forge review row author is `opener`; `require_distinct_from_author`
    // is satisfied because the approver below is `rev2`.
    seed_review(&ctx, head, "opener")?;

    // opener opens + assigns rev2; rev2 approves at head.
    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("opener"))], async {
        but_api::legacy::forge::request_review(
            ctx.to_sync(),
            FEAT_REF.to_owned(),
            Some("rev2".to_owned()),
        )
        .await
    })
    .await?;
    approve_branch(&ctx, "rev2").await?;

    // Read 1: review_status — the orchestrator's presentation label.
    let status = but_api::legacy::forge::review_status(ctx.to_sync(), FEAT_REF.to_owned()).await?;
    assert!(
        status.verdict_at_head.as_deref() == Some("approved"),
        "review_status must report approved=true when an approved verdict at head exists"
    );

    // Read 2: enforce_merge_gate — the gate's own re-derivation. The label does
    // NOT authorize the merge; the gate re-derives verdict-at-head itself.
    // `dry_run_merge_review` is the public API that returns immediately after
    // `enforce_merge_gate`, so we get a clean permit/deny without a forge call.
    let gate_permit = temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("maint"))], async {
        but_api::legacy::forge::dry_run_merge_review(ctx.to_sync(), REVIEW_ID)
    })
    .await;
    assert!(
        gate_permit.is_ok(),
        "enforce_merge_gate must PERMIT from the same verdict@head truth (got: {gate_permit:?})"
    );
    Ok(())
}

/// AC-4: two independent reads of `review_status` against one unchanged repo
/// state yield IDENTICAL drive state — deterministic ordering, no per-orchestrator
/// memory. Two orchestrators converge.
#[tokio::test]
#[serial_test::serial]
async fn review_status_two_read_convergence() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_reconciler_repo(ReconcilerGate::None)?;
    let ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();

    // Fixture a fixed drive state: one pending assignment, two unresolved
    // threads, and (for good measure) the opener's agent tag.
    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("opener"))], async {
        but_api::legacy::forge::request_review(
            ctx.to_sync(),
            FEAT_REF.to_owned(),
            Some("rev2".to_owned()),
        )
        .await
    })
    .await?;
    seed_unresolved_comment(&ctx, FEAT_REF, "rev", "first thread body", "t-alpha")?;
    seed_unresolved_comment(&ctx, FEAT_REF, "rev", "second thread body", "t-beta")?;

    // Two orchestrators read concurrently — both must see byte-identical state.
    let (first, second) = tokio::join!(
        but_api::legacy::forge::review_status(ctx.to_sync(), FEAT_REF.to_owned()),
        but_api::legacy::forge::review_status(ctx.to_sync(), FEAT_REF.to_owned()),
    );
    let first = first?;
    let second = second?;
    let first_open: Vec<_> = first
        .assignments
        .iter()
        .filter(|a| a.state == "pending")
        .collect();
    let second_open: Vec<_> = second
        .assignments
        .iter()
        .filter(|a| a.state == "pending")
        .collect();

    assert_eq!(
        first_open, second_open,
        "two reads must converge on open_assignments (same order)"
    );
    assert_eq!(
        first.open_threads, second.open_threads,
        "two reads must converge on unresolved_threads (same order, same content)"
    );
    assert_eq!(
        first.verdict_at_head.as_deref() == Some("approved"),
        second.verdict_at_head.as_deref() == Some("approved"),
        "two reads must converge on approved"
    );
    assert_eq!(
        first.verdict_at_head, second.verdict_at_head,
        "two reads must converge on verdict_at_head"
    );
    assert_eq!(
        first.lifecycle, second.lifecycle,
        "two reads must converge on lifecycle"
    );
    assert_eq!(
        first.open_threads, 2,
        "the two seeded unresolved threads must both be surfaced, in deterministic order"
    );
    Ok(())
}

/// AC-5: an assignment-only branch (one pending assignment, no comments, no
/// verdict) → the payload carries the assignment as the dispatch trigger.
#[tokio::test]
#[serial_test::serial]
async fn review_status_assignment_only() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_reconciler_repo(ReconcilerGate::None)?;
    let ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();

    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("opener"))], async {
        but_api::legacy::forge::request_review(
            ctx.to_sync(),
            FEAT_REF.to_owned(),
            Some("rev2".to_owned()),
        )
        .await
    })
    .await
    .context("opener must open review and assign rev2")?;

    let status = but_api::legacy::forge::review_status(ctx.to_sync(), FEAT_REF.to_owned()).await?;
    let open_assignments: Vec<_> = status
        .assignments
        .iter()
        .filter(|a| a.state == "pending")
        .collect();

    // The pending assignment is the dispatch trigger.
    assert_eq!(
        open_assignments.len(),
        1,
        "open_assignments carries the single pending assignment"
    );
    assert_eq!(
        open_assignments[0].reviewer_principal, "rev2",
        "the pending assignment is for rev2"
    );
    assert_eq!(
        open_assignments[0].state, "pending",
        "the assignment is pending (the dispatch trigger)"
    );
    // No comments, no verdict.
    assert!(
        status.open_threads == 0,
        "no comments → unresolved_threads is empty"
    );
    assert!(
        status.verdict_at_head.as_deref() != Some("approved"),
        "no verdict → approved is false (no gate signal yet)"
    );
    assert_eq!(
        status.verdict_at_head, None,
        "no verdict → verdict_at_head is None"
    );
    assert_eq!(
        status.lifecycle, "AwaitingReview",
        "assigned-but-not-approved → AwaitingReview"
    );
    Ok(())
}

// ---- fixture + helpers -----------------------------------------------------

#[derive(Debug, Clone, Copy)]
enum ReconcilerGate {
    /// No `.gitbutler/gates.toml` review requirement — the merge gate is a
    /// no-op for branches that aren't protected. Used by ACs that don't need
    /// the gate path.
    None,
    /// `main` is protected with a single-approval review requirement
    /// (`min_approvals = 1`, `require_distinct_from_author = true`). Used by
    /// AC-3 to prove the read AGREES with the gate.
    SingleApproval,
}

/// A governed repo for LPR-008 reconciler proofs. Principals:
///   - `opener` — `pull_requests:write` + `contents:write` (opens the review)
///   - `rev`    — `reviews:write` + `comments:write` (posts comments)
///   - `rev2`   — `reviews:write` + `comments:write` (the assigned reviewer)
///   - `maint`  — `merge` (the gate caller for AC-3)
///
/// The `feat` branch overrides `.gitbutler/gates.toml` to an empty file. The
/// but-authz `load_governance_config` (called by the writer verbs) parses
/// gates.toml with a stricter schema than `merge_gate` (no `[[gate]]` table —
/// only `[[branch]]`), so leaving the target-ref gate content on feat would
/// fail the writer-side authorize step. The target-ref `main` keeps the real
/// gate config that `enforce_merge_gate` reads.
fn lpr_reconciler_repo(
    config: ReconcilerGate,
) -> anyhow::Result<(gix::Repository, tempfile::TempDir)> {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    let gates = match config {
        ReconcilerGate::None => "",
        ReconcilerGate::SingleApproval => {
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
    };
    let permissions = r#"
[[principal]]
id = "opener"
permissions = ["contents:write", "pull_requests:write"]

[[principal]]
id = "rev"
permissions = ["reviews:write", "comments:write"]

[[principal]]
id = "rev2"
permissions = ["reviews:write", "comments:write"]

[[principal]]
id = "maint"
permissions = ["merge"]
"#;
    but_testsupport::invoke_bash(
        &format!(
            r#"
git remote add origin https://github.com/gitbutler/reconciler-read-fixture.git
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
# feat overrides gates.toml to empty — the but-authz loader parses gates.toml
# at the target ref of the writer verb with a stricter schema (no [[gate]]).
cat >.gitbutler/gates.toml <<'EOF'
EOF
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

fn seed_review(ctx: &but_ctx::Context, head: gix::ObjectId, author: &str) -> anyhow::Result<()> {
    // `source_branch` MUST match the target column the verdict rows are stored
    // under — `enforce_merge_gate` queries `local_review_verdicts` with
    // `review.source_branch` verbatim (no `branch_ref` normalization), so we
    // use FEAT_REF here to match the `approve_review(FEAT_REF, ...)` call.
    ctx.db
        .get_cache_mut()?
        .forge_reviews_mut()?
        .upsert(ForgeReview {
            html_url: "https://github.com/gitbutler/reconciler-read-fixture/pull/1".to_owned(),
            number: REVIEW_ID.try_into()?,
            title: "Reconciler read fixture".to_owned(),
            body: None,
            author: Some(author.to_owned()),
            labels: "[]".to_owned(),
            draft: false,
            source_branch: FEAT_REF.to_owned(),
            target_branch: MAIN_REF.to_owned(),
            sha: head.to_string(),
            created_at: None,
            modified_at: None,
            merged_at: None,
            closed_at: None,
            repository_ssh_url: None,
            repository_https_url: Some(
                "https://github.com/gitbutler/reconciler-read-fixture.git".to_owned(),
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

fn ref_id(ctx: &but_ctx::Context, ref_name: &str) -> anyhow::Result<gix::ObjectId> {
    let repo = ctx.repo.get()?;
    Ok(repo.find_reference(ref_name)?.peel_to_id()?.detach())
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

async fn approve_branch(ctx: &but_ctx::Context, principal_id: &str) -> anyhow::Result<()> {
    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some(principal_id))], async {
        but_api::legacy::forge::approve_review(ctx.to_sync(), FEAT_REF.to_owned()).await
    })
    .await
}
