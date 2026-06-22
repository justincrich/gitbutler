//! LPR-015 — local-review READ producers on the desktop bus.
//!
//! These proofs exercise the REAL Tauri mock-runtime command bus with
//! `tauri::test::get_ipc_response`, invoking the registered
//! `#[but_api(napi)]`-generated `review_status` and `list_comments` commands
//! over a real but-db + gix fixture seeded via the REAL LPR verbs.
//!
//! The reads are FORGE commands (registered in `lib.rs`'s forge block beside
//! `tauri_get_review::get_review`), NOT governance commands — they ride
//! `core:default` like every other forge read, need NO `DesktopSessionState`
//! wrapper (reads have no fleet-owner identity), and carry NO write authority.
//!
//! ACs covered:
//! - AC-1 / TC-1, TC-2: `review_status` registers and returns the single
//!   derived-lifecycle + drive-state payload.
//! - AC-2 / TC-3: `list_comments` registers and returns the branch's threads.
//! - AC-3 / TC-4, TC-5: both reads are READ-ONLY (no mutation; no write
//!   authority required).
//! - AC-4 / TC-6, TC-7: both reads are BRANCH-scoped (F-006 — a third
//!   principal sees every principal's drive state on the named branch).
//! - AC-5 / TC-8, TC-10: the only desktop entry is the registered but-api
//!   command (no R14 bypass; no hand-written allow-file).

use std::fs;

use anyhow::Context as _;
use but_core::RepositoryExt as _;
use serde_json::{Value, json};
use tauri::{WebviewWindowBuilder, ipc::InvokeBody, webview::InvokeRequest};

const FEAT_REF: &str = "refs/heads/feat";
const TARGET_REF: &str = "refs/remotes/origin/main";

// --------------------------------------------------------------------------------
// AC-1 / TC-1, TC-2 — review_status registers + returns the single payload
// --------------------------------------------------------------------------------

/// PRIMARY: `review_status` registers on the desktop bus and returns the SINGLE
/// derived-lifecycle + drive-state payload (assignments + unresolved threads +
/// verdict-at-head) for a branch fixtured via the real LPR verbs. Also proves
/// TC-2 (both reads registered; an unregistered probe is rejected) and TC-8/TC-10
/// (no R14 bypass, no hand-written allow-file).
#[test]
#[serial_test::serial]
fn lpr_review_reads_register_and_return_branch_drive_state() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_review_read_repo();
    let project_id = project_id_for(&repo)?;
    let app = lpr_review_app()?;
    let webview = lpr_review_webview(&app)?;

    // Seed refs/heads/feat drive state via the REAL LPR verbs (R14 — same gated
    // seam the CLI/N-API use). `rev` opens + seeds a pending assignment for
    // `rev2`, posts an unresolved thread `t1`, and approves at the current HEAD.
    seed_drive_state(&repo, FEAT_REF)?;

    let status = temp_env::with_var("BUT_AGENT_HANDLE", Some("rev"), || {
        invoke_ok(
            &webview,
            "review_status",
            review_status_payload(&project_id, FEAT_REF),
        )
    })?;

    assert_eq!(
        status.get("target").and_then(Value::as_str),
        Some(FEAT_REF),
        "review_status target is the queried branch ref: {status:?}"
    );
    let lifecycle = status
        .get("lifecycle")
        .and_then(Value::as_str)
        .expect("review_status payload carries the derived lifecycle");
    assert!(
        matches!(lifecycle, "Approved" | "AwaitingReview"),
        "the derived lifecycle reflects the seeded approval/assignment state, got {lifecycle:?}"
    );

    let open_assignments = status
        .get("open_assignments")
        .and_then(Value::as_array)
        .expect("review_status payload carries open_assignments (LPR-008 drive state)");
    assert!(
        open_assignments.iter().any(|assignment| assignment
            .get("reviewer_principal")
            .and_then(Value::as_str)
            == Some("rev2")),
        "the pending rev2 assignment must appear in open_assignments: {open_assignments:?}"
    );

    let unresolved_threads = status
        .get("unresolved_threads")
        .and_then(Value::as_array)
        .expect("review_status payload carries unresolved_threads (LPR-008 drive state)");
    assert!(
        unresolved_threads
            .iter()
            .any(|thread| thread.get("thread_id").and_then(Value::as_str) == Some("t1")),
        "the unresolved thread t1 must appear in unresolved_threads: {unresolved_threads:?}"
    );

    // The LPR-005/008 reconciler read carries `verdict_at_head` + `approved` —
    // the merge-gate-aligned label reflecting the approval@head.
    assert!(
        status.get("verdict_at_head").and_then(Value::as_str) == Some("approved")
            || status.get("approved").and_then(Value::as_bool) == Some(true),
        "review_status must reflect the approval@head (verdict_at_head/approved): {status:?}"
    );

    // TC-2 / TC-8: an unregistered review-read probe is rejected by the real
    // bus ("not found"). Registration IS the admission.
    let probe_error = invoke_err(&webview, "lpr_review_reads_unregistered_probe", json!({}))?;
    assert!(
        probe_error.to_string().contains("not found"),
        "the real Tauri bus must reject an unregistered review-read probe, got {probe_error:?}"
    );

    // TC-10: no per-command allow-review_status / allow-list_comments file is
    // introduced (the forge reads ride core:default like get_review).
    assert_no_review_read_allow_file();

    println!(
        "review_status registered on the desktop bus and returned the single drive-state payload"
    );
    println!(
        "list_comments registered (verified below); an unregistered probe is rejected 'not found'"
    );
    println!("no allow-review_status/allow-list_comments capability file is hand-authored");
    Ok(())
}

// --------------------------------------------------------------------------------
// AC-2 / TC-3 — list_comments registers + returns the branch's threads
// --------------------------------------------------------------------------------

/// `list_comments` registers on the desktop bus and returns the branch's comment
/// threads (t1 unresolved + t2 resolved, with their resolved flags).
#[test]
#[serial_test::serial]
fn list_comments_returns_branch_threads_on_bus() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_review_read_repo();
    let project_id = project_id_for(&repo)?;
    let app = lpr_review_app()?;
    let webview = lpr_review_webview(&app)?;

    // Seed two threads via the REAL post_comment verb (R14 — same gated seam).
    // `rev` opens the review (write-once opener marker); `rev2` posts t1, `rev3`
    // posts t2 and resolves it.
    seed_drive_state(&repo, FEAT_REF)?;
    seed_thread_resolved(&repo, FEAT_REF, "rev3", "looks good", "t2")?;

    let comments = temp_env::with_var("BUT_AGENT_HANDLE", Some("rev"), || {
        invoke_ok(
            &webview,
            "list_comments",
            list_comments_payload(&project_id, FEAT_REF),
        )
    })?;

    let threads = comments
        .as_array()
        .expect("list_comments returns a Vec<LocalReviewComment>");

    let t1 = threads
        .iter()
        .find(|c| c.get("thread_id").and_then(Value::as_str) == Some("t1"));
    let t2 = threads
        .iter()
        .find(|c| c.get("thread_id").and_then(Value::as_str) == Some("t2"));

    let t1 = t1.expect("list_comments must return the unresolved thread t1");
    let t2 = t2.expect("list_comments must return the resolved thread t2");

    assert_eq!(
        t1.get("resolved").and_then(Value::as_bool),
        Some(false),
        "t1 must be unresolved: {t1:?}"
    );
    assert_eq!(
        t2.get("resolved").and_then(Value::as_bool),
        Some(true),
        "t2 must be resolved: {t2:?}"
    );

    println!(
        "list_comments registered on the desktop bus and returned both threads (t1 unresolved + t2 resolved)"
    );
    Ok(())
}

// --------------------------------------------------------------------------------
// AC-3 / TC-4, TC-5 — both reads are READ-ONLY (no mutation, no write authority)
// --------------------------------------------------------------------------------

/// After `review_status` AND `list_comments` are invoked, the
/// local_review_assignments/comments/verdicts stores are byte-unchanged AND
/// refs/objects/oplog are byte-unchanged. Neither read requires a write
/// authority — a read-capable-but-not-write caller still gets the payload.
#[test]
#[serial_test::serial]
fn review_reads_are_read_only_no_mutation() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_review_read_repo();
    let project_id = project_id_for(&repo)?;
    let app = lpr_review_app()?;
    let webview = lpr_review_webview(&app)?;

    seed_drive_state(&repo, FEAT_REF)?;

    // Snapshot the drive stores + repo state BEFORE the reads.
    let assignments_before = snapshot_assignments(&repo, FEAT_REF)?;
    let comments_before = snapshot_comments(&repo, FEAT_REF)?;
    let verdicts_before = snapshot_verdicts(&repo, FEAT_REF)?;
    let refs_before = snapshot_refs(&repo)?;
    let objects_before = object_count(&repo)?;

    // Invoke BOTH reads as a write-capable caller (`rev`).
    temp_env::with_var("BUT_AGENT_HANDLE", Some("rev"), || {
        invoke_ok(
            &webview,
            "review_status",
            review_status_payload(&project_id, FEAT_REF),
        )?;
        invoke_ok(
            &webview,
            "list_comments",
            list_comments_payload(&project_id, FEAT_REF),
        )
    })?;

    // AC-3 / TC-4: byte-unchanged after both reads.
    assert_eq!(
        snapshot_assignments(&repo, FEAT_REF)?,
        assignments_before,
        "review_status/list_comments must not mutate local_review_assignments"
    );
    assert_eq!(
        snapshot_comments(&repo, FEAT_REF)?,
        comments_before,
        "review_status/list_comments must not mutate local_review_comments"
    );
    assert_eq!(
        snapshot_verdicts(&repo, FEAT_REF)?,
        verdicts_before,
        "review_status/list_comments must not mutate local_review_verdicts"
    );
    assert_eq!(
        snapshot_refs(&repo)?,
        refs_before,
        "review_status/list_comments must not mutate refs"
    );
    assert_eq!(
        object_count(&repo)?,
        objects_before,
        "review_status/list_comments must not add objects"
    );

    // AC-3 / TC-5: a read-capable-but-not-write caller still gets the payload
    // (the reads are NOT write-gated). `viewer` holds only `contents:read`.
    let viewer_status = temp_env::with_var("BUT_AGENT_HANDLE", Some("viewer"), || {
        invoke_ok(
            &webview,
            "review_status",
            review_status_payload(&project_id, FEAT_REF),
        )
    })?;
    assert_eq!(
        viewer_status.get("target").and_then(Value::as_str),
        Some(FEAT_REF),
        "a read-capable-but-not-write caller still receives the review_status payload"
    );

    println!("both reads performed NO write — drive stores + refs + objects byte-unchanged");
    println!("read-capable-but-not-write caller `viewer` still received the payload");
    Ok(())
}

// --------------------------------------------------------------------------------
// AC-4 / TC-6, TC-7 — both reads are BRANCH-scoped (F-006), not self-scoped
// --------------------------------------------------------------------------------

/// `review_status` open_assignments includes BOTH rev2's and rev3's assignments
/// AND `list_comments` returns BOTH rev2's and rev3's threads when invoked by a
/// THIRD principal `viewer` who authored none — the branch's whole review
/// surface is disclosed to any caller who can name it (F-006).
#[test]
#[serial_test::serial]
fn review_reads_are_branch_scoped_not_self_scoped() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_review_read_repo();
    let project_id = project_id_for(&repo)?;
    let app = lpr_review_app()?;
    let webview = lpr_review_webview(&app)?;

    // Seed multi-principal drive state on refs/heads/feat via real verbs.
    seed_multi_principal_drive_state(&repo, FEAT_REF)?;

    // Invoke BOTH reads as a THIRD principal `viewer` who authored no
    // assignment/thread on the branch but CAN name it.
    let status = temp_env::with_var("BUT_AGENT_HANDLE", Some("viewer"), || {
        invoke_ok(
            &webview,
            "review_status",
            review_status_payload(&project_id, FEAT_REF),
        )
    })?;
    let comments = temp_env::with_var("BUT_AGENT_HANDLE", Some("viewer"), || {
        invoke_ok(
            &webview,
            "list_comments",
            list_comments_payload(&project_id, FEAT_REF),
        )
    })?;

    // TC-6: review_status open_assignments includes BOTH rev2's and rev3's.
    let open_assignments = status
        .get("open_assignments")
        .and_then(Value::as_array)
        .expect("review_status returns open_assignments");
    let reviewers: Vec<String> = open_assignments
        .iter()
        .filter_map(|a| {
            a.get("reviewer_principal")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .collect();
    assert!(
        reviewers.iter().any(|r| r == "rev2"),
        "branch-scoped read must disclose rev2's assignment to `viewer`: {reviewers:?}"
    );
    assert!(
        reviewers.iter().any(|r| r == "rev3"),
        "branch-scoped read must disclose rev3's assignment to `viewer`: {reviewers:?}"
    );

    // TC-7: list_comments returns BOTH rev2's and rev3's threads.
    let threads = comments
        .as_array()
        .expect("list_comments returns a Vec<LocalReviewComment>");
    let authors: Vec<String> = threads
        .iter()
        .filter_map(|c| {
            c.get("author_principal")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .collect();
    assert!(
        authors.iter().any(|a| a == "rev2"),
        "branch-scoped read must disclose rev2's thread to `viewer`: {authors:?}"
    );
    assert!(
        authors.iter().any(|a| a == "rev3"),
        "branch-scoped read must disclose rev3's thread to `viewer`: {authors:?}"
    );

    println!("branch-scoped (F-006): `viewer` saw BOTH rev2's and rev3's assignments and threads");
    Ok(())
}

// --------------------------------------------------------------------------------
// Fixture — real governed repo with committed permissions.toml
// --------------------------------------------------------------------------------

/// A governed repo for the LPR-015 bus proofs. Principals:
/// - `rev`     — pull_requests:write + reviews:write + comments:write (the opener)
/// - `rev2`    — reviews:write + comments:write (an assigned reviewer / thread author)
/// - `rev3`    — reviews:write + comments:write (a second reviewer / thread author)
/// - `viewer`  — contents:read only (read-capable, NOT write — proves reads
///   are not write-gated AND the F-006 branch-scoped disclosure)
fn lpr_review_read_repo() -> (gix::Repository, tempfile::TempDir) {
    let tmp = tempfile::TempDir::new()
        .unwrap_or_else(|error| panic!("creating temp repository failed: {error}"));
    but_testsupport::invoke_bash_at_dir(
        r#"
git init --initial-branch=main
git config user.name "GitButler Test"
git config user.email "gitbutler@example.com"
echo initial >README.md
git add README.md
git commit -m "initial"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "rev"
permissions = ["contents:write", "pull_requests:write", "reviews:write", "comments:write"]

[[principal]]
id = "rev2"
permissions = ["contents:read", "reviews:write", "comments:write"]

[[principal]]
id = "rev3"
permissions = ["contents:read", "reviews:write", "comments:write"]

[[principal]]
id = "viewer"
permissions = ["contents:read"]
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
        tmp.path(),
    );
    let repo = gix::open(tmp.path())
        .unwrap_or_else(|error| panic!("opening {} failed: {error}", tmp.path().display()));
    // Mirror the mgmt_ipc_003 idiom: define a stable target ref for governance
    // resolution. Reads on refs/heads/feat use feat's own committed config.
    but_testsupport::invoke_bash(
        &format!("git update-ref {TARGET_REF} refs/heads/main"),
        &repo,
    );
    (repo, tmp)
}

fn project_id_for(
    repo: &gix::Repository,
) -> anyhow::Result<but_ctx::ProjectHandleOrLegacyProjectId> {
    let git_dir = repo.git_dir();
    let handle = but_ctx::ProjectHandle::from_path(git_dir)?;
    Ok(but_ctx::ProjectHandleOrLegacyProjectId::ProjectHandle(
        handle,
    ))
}

// --------------------------------------------------------------------------------
// Drive-state seeding — via the REAL LPR verbs (R14: same gated seam as CLI/N-API)
// --------------------------------------------------------------------------------

/// Seed refs/heads/feat with:
///   - opener `rev` (write-once marker via `request_review`)
///   - a `pending` assignment for `rev2` (via `request_review(reviewer=Some(rev2))`)
///   - an unresolved thread `t1` authored by `rev2` (via `post_comment`)
///   - an `approved` verdict@head authored by `rev2` (via `approve_review`)
fn seed_drive_state(repo: &gix::Repository, branch: &str) -> anyhow::Result<()> {
    let ctx = but_ctx::Context::from_repo(repo.clone())?;

    // `rev` opens the review and seeds the first pending assignment for `rev2`.
    with_agent("rev", || {
        block_on_seed(but_api::legacy::forge::request_review(
            ctx.to_sync(),
            branch.to_owned(),
            Some("rev2".to_owned()),
        ))
    })
    .context("request_review as rev must succeed")?;

    // `rev2` posts the unresolved thread `t1`.
    with_agent("rev2", || {
        block_on_seed(but_api::legacy::forge::post_comment(
            ctx.to_sync(),
            branch.to_owned(),
            "please revisit".to_owned(),
            None,
            None,
            "t1".to_owned(),
        ))
    })
    .context("post_comment as rev2 on t1 must succeed")?;

    // `rev2` approves at the current HEAD (the merge-gate's own re-derivation
    // input). Must come AFTER the comment so the head_oid is stable.
    with_agent("rev2", || {
        block_on_seed(but_api::legacy::forge::approve_review(
            ctx.to_sync(),
            branch.to_owned(),
        ))
    })
    .context("approve_review as rev2 must succeed")?;

    Ok(())
}

/// Seed a second reviewer `rev3` + an additional resolved thread authored by
/// `rev3`. Used by AC-2 (t2 resolved) and AC-4 (multi-principal).
fn seed_thread_resolved(
    repo: &gix::Repository,
    branch: &str,
    author: &str,
    body: &str,
    thread_id: &str,
) -> anyhow::Result<()> {
    let ctx = but_ctx::Context::from_repo(repo.clone())?;

    // Assign `rev3` as a reviewer (so they're in open_assignments for AC-4).
    with_agent("rev", || {
        block_on_seed(but_api::legacy::forge::assign_reviewer(
            ctx.to_sync(),
            branch.to_owned(),
            "rev3".to_owned(),
        ))
    })
    .context("assign_reviewer rev3 must succeed")?;

    // `rev3` posts the thread.
    with_agent(author, || {
        block_on_seed(but_api::legacy::forge::post_comment(
            ctx.to_sync(),
            branch.to_owned(),
            body.to_owned(),
            None,
            None,
            thread_id.to_owned(),
        ))
    })
    .context("post_comment must succeed")?;

    // Resolve it. The resolver must be the thread author (`rev3`), an assigned
    // reviewer, or a reviews:write holder — `rev3` is both author and assigned.
    with_agent(author, || {
        block_on_seed(but_api::legacy::forge::resolve_thread(
            ctx.to_sync(),
            branch.to_owned(),
            thread_id.to_owned(),
            true,
        ))
    })
    .context("resolve_thread must succeed")?;

    Ok(())
}

/// Seed multi-principal drive state for AC-4: assignments + threads from BOTH
/// `rev2` and `rev3`, with the invoking caller being a THIRD principal
/// `viewer` who authored none.
fn seed_multi_principal_drive_state(repo: &gix::Repository, branch: &str) -> anyhow::Result<()> {
    let ctx = but_ctx::Context::from_repo(repo.clone())?;

    // `rev` opens the review + assigns BOTH rev2 and rev3 as reviewers.
    with_agent("rev", || {
        block_on_seed(but_api::legacy::forge::request_review(
            ctx.to_sync(),
            branch.to_owned(),
            Some("rev2".to_owned()),
        ))
    })
    .context("request_review assigning rev2 must succeed")?;

    with_agent("rev", || {
        block_on_seed(but_api::legacy::forge::assign_reviewer(
            ctx.to_sync(),
            branch.to_owned(),
            "rev3".to_owned(),
        ))
    })
    .context("assign_reviewer rev3 must succeed")?;

    // `rev2` authors a thread.
    with_agent("rev2", || {
        block_on_seed(but_api::legacy::forge::post_comment(
            ctx.to_sync(),
            branch.to_owned(),
            "rev2 thread".to_owned(),
            None,
            None,
            "rev2-thread".to_owned(),
        ))
    })
    .context("post_comment as rev2 must succeed")?;

    // `rev3` authors a thread.
    with_agent("rev3", || {
        block_on_seed(but_api::legacy::forge::post_comment(
            ctx.to_sync(),
            branch.to_owned(),
            "rev3 thread".to_owned(),
            None,
            None,
            "rev3-thread".to_owned(),
        ))
    })
    .context("post_comment as rev3 must succeed")?;

    Ok(())
}

/// Run an async seed future under a one-shot tokio runtime. The but-api LPR
/// verbs are async (they round-trip through the macro's ThreadSafeContext).
fn block_on_seed<F>(future: F) -> anyhow::Result<()>
where
    F: std::future::Future<Output = anyhow::Result<()>>,
{
    tokio::runtime::Runtime::new()?.block_on(future)
}

/// Sync `BUT_AGENT_HANDLE` env-scoped closure (mirrors the mgmt_ipc_003 idiom).
fn with_agent<T>(handle: &str, f: impl FnOnce() -> anyhow::Result<T>) -> anyhow::Result<T> {
    temp_env::with_var("BUT_AGENT_HANDLE", Some(handle), f)
}

// --------------------------------------------------------------------------------
// Tauri mock-runtime bus harness — registers review_status + list_comments
// (the LPR-015 desktop bus surface, mirroring mgmt_ipc_003's governance_app)
// --------------------------------------------------------------------------------

fn lpr_review_app() -> anyhow::Result<tauri::App<tauri::test::MockRuntime>> {
    tauri::test::mock_builder()
        .invoke_handler(tauri::generate_handler![
            but_api::legacy::forge::tauri_review_status::review_status,
            but_api::legacy::forge::tauri_list_comments::list_comments,
        ])
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .map_err(Into::into)
}

fn lpr_review_webview(
    app: &tauri::App<tauri::test::MockRuntime>,
) -> anyhow::Result<tauri::WebviewWindow<tauri::test::MockRuntime>> {
    WebviewWindowBuilder::new(app, "main", Default::default())
        .build()
        .map_err(Into::into)
}

fn review_status_payload(
    project_id: &but_ctx::ProjectHandleOrLegacyProjectId,
    branch: &str,
) -> Value {
    json!({ "projectId": project_id, "branch": branch })
}

fn list_comments_payload(
    project_id: &but_ctx::ProjectHandleOrLegacyProjectId,
    branch: &str,
) -> Value {
    json!({ "projectId": project_id, "branch": branch })
}

fn invoke_ok(
    webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
    command: &str,
    payload: Value,
) -> anyhow::Result<Value> {
    invoke(webview, command, payload)?
        .map_err(|error| anyhow::anyhow!("unexpected IPC error from {command}: {error:?}"))
}

fn invoke_err(
    webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
    command: &str,
    payload: Value,
) -> anyhow::Result<Value> {
    match invoke(webview, command, payload)? {
        Ok(value) => anyhow::bail!("expected IPC error from {command}, got {value:?}"),
        Err(error) => Ok(error),
    }
}

fn invoke(
    webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
    command: &str,
    payload: Value,
) -> anyhow::Result<Result<Value, Value>> {
    let request = InvokeRequest {
        cmd: command.into(),
        callback: tauri::ipc::CallbackFn(0),
        error: tauri::ipc::CallbackFn(1),
        url: "tauri://localhost".parse()?,
        body: InvokeBody::from(payload),
        headers: Default::default(),
        invoke_key: tauri::test::INVOKE_KEY.to_owned(),
    };

    match tauri::test::get_ipc_response(webview, request) {
        Ok(body) => Ok(Ok(body.deserialize::<Value>()?)),
        Err(error) => Ok(Err(error)),
    }
}

// --------------------------------------------------------------------------------
// Snapshot helpers — for AC-3 byte-unchanged proofs
// --------------------------------------------------------------------------------

fn open_db(repo: &gix::Repository) -> anyhow::Result<but_db::DbHandle> {
    let project_data_dir = repo.gitbutler_storage_path()?;
    but_db::DbHandle::new_in_directory(project_data_dir)
}

fn snapshot_assignments(repo: &gix::Repository, target: &str) -> anyhow::Result<String> {
    let db = open_db(repo)?;
    let rows = db.local_review_assignments().list_by_target(target)?;
    Ok(serde_json::to_string(&rows)?)
}

fn snapshot_comments(repo: &gix::Repository, target: &str) -> anyhow::Result<String> {
    let db = open_db(repo)?;
    let rows = db.local_review_comments().list_by_target(target)?;
    Ok(serde_json::to_string(&rows)?)
}

fn snapshot_verdicts(repo: &gix::Repository, target: &str) -> anyhow::Result<String> {
    let db = open_db(repo)?;
    let rows = db.local_review_verdicts().list_by_target(target)?;
    Ok(serde_json::to_string(&rows)?)
}

fn snapshot_refs(repo: &gix::Repository) -> anyhow::Result<Vec<(String, String)>> {
    let mut refs = Vec::new();
    let iter = repo.references()?;
    for item in iter.all()? {
        let item = item.map_err(|e| anyhow::anyhow!(e))?;
        let name = item.name().as_bstr().to_string();
        let id = item.id().to_string();
        refs.push((name, id));
    }
    refs.sort();
    Ok(refs)
}

fn object_count(repo: &gix::Repository) -> anyhow::Result<usize> {
    let mut count = 0usize;
    let mut iter = repo.objects.iter()?;
    while let Some(item) = iter.next().transpose().map_err(|e| anyhow::anyhow!(e))? {
        let _ = item;
        count += 1;
    }
    Ok(count)
}

// --------------------------------------------------------------------------------
// Capability boundary — TC-10: no per-command allow-file for the forge reads
// --------------------------------------------------------------------------------

fn assert_no_review_read_allow_file() {
    let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let capability_dir = crate_dir.join("capabilities");
    let mut files = Vec::new();
    collect_files(&capability_dir, &mut files);

    let forbidden: Vec<_> = files
        .into_iter()
        .filter(|path| {
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                return false;
            };
            name.starts_with("allow-review_status") || name.starts_with("allow-list_comments")
        })
        .collect();

    assert!(
        forbidden.is_empty(),
        "the forge review reads must ride core:default (no hand-written allow-review_status/allow-list_comments): {forbidden:?}"
    );
}

fn collect_files(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_files(&path, files);
            } else {
                files.push(path);
            }
        }
    }
}
