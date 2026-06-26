//! LPR-003 integration proofs (AC-1..AC-5) — real but-db + real gix.
//!
//! Mirrors the `forge_guard` hand-assertion idiom: a governed scenario repo
//! (committed `.gitbutler/permissions.toml` + `.gitbutler/gates.toml`),
//! `temp_env::with_var("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")), ("BUT_AGENT_HANDLE", ...)` to switch principals, drive
//! the real `but_api::legacy::forge::{request_review, assign_reviewer,
//! request_changes_review}` verbs, and assert directly against
//! `local_review_assignments` / `local_review_meta` / `local_review_verdicts`.
//!
//! No mocks, no insta — real but-db + real gix only.

use anyhow::Context as _;

const FEAT_REF: &str = "refs/heads/feat";

#[test]
#[serial_test::serial]
fn request_review_persists_pending_assignment_without_touching_verdicts() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_governed_repo();
    let ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();
    let runtime = tokio::runtime::Runtime::new()?;

    let verdicts_before = ctx
        .db
        .get_cache()?
        .local_review_verdicts()
        .list_by_target(FEAT_REF)?;
    assert!(
        verdicts_before.is_empty(),
        "fixture must seed an empty verdict store"
    );

    // `dev` holds pull_requests:write — the open-PR authority.
    temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("dev")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || {
            runtime.block_on(but_api::legacy::forge::request_review(
                ctx.to_sync(),
                FEAT_REF.to_owned(),
                Some("rev2".to_owned()),
            ))
        },
    )
    .context("request_review as dev (pull_requests:write) must succeed")?;

    let db = ctx.db.get_cache()?;

    let assignments = db.local_review_assignments().list_by_target(FEAT_REF)?;
    assert_eq!(
        assignments.len(),
        1,
        "request_review must write exactly one pending assignment row"
    );
    let [assignment] = assignments.as_slice() else {
        unreachable!("len asserted as one, so first assignment exists");
    };
    assert_eq!(assignment.target, FEAT_REF);
    assert_eq!(
        assignment.reviewer_principal, "rev2",
        "assignment reviewer must be the requested reviewer principal"
    );
    assert_eq!(
        assignment.state,
        but_authz::AssignmentState::Pending.name(),
        "assignment state must be `pending` via AssignmentState::Pending.name()"
    );

    let opener = db
        .local_review_meta()
        .get(FEAT_REF, "opener_principal")?
        .context("request_review must write the write-once local_review_meta opener row")?;
    assert_eq!(
        opener.value, "dev",
        "opener_principal value must be the authorized caller principal, not a caller-supplied flag"
    );
    assert_eq!(opener.target, FEAT_REF);
    assert_eq!(opener.key, "opener_principal");

    let verdicts_after = db.local_review_verdicts().list_by_target(FEAT_REF)?;
    assert!(
        verdicts_after.is_empty(),
        "request_review must NOT touch local_review_verdicts — assignments and verdicts are separate"
    );

    Ok(())
}

#[test]
#[serial_test::serial]
fn assign_reviewer_distinct_from_author_upserts_idempotent() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_governed_repo();
    let ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();
    let runtime = tokio::runtime::Runtime::new()?;

    // Establish `auth` as the target branch author principal by recording it as
    // the opener (the canonical source for "target author" at the but-api
    // boundary). `dev` opens the review on PullRequestsWrite.
    temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("auth")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || {
            runtime.block_on(but_api::legacy::forge::request_review(
                ctx.to_sync(),
                FEAT_REF.to_owned(),
                None,
            ))
        },
    )
    .context("request_review as auth establishes the opener principal")?;

    // `rev` holds reviews:write — the assignment authority.
    let assign = |reviewer: &str| {
        temp_env::with_vars(
            [
                ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
                ("BUT_AGENT_HANDLE", Some("rev")),
                ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ],
            || {
                runtime.block_on(but_api::legacy::forge::assign_reviewer(
                    ctx.to_sync(),
                    FEAT_REF.to_owned(),
                    reviewer.to_owned(),
                ))
            },
        )
    };

    assign("rev2").context("first assign_reviewer(rev2) must succeed")?;
    assign("rev2").context("second assign_reviewer(rev2) must be idempotent")?;
    assign("rev3").context("assign_reviewer(rev3) must succeed (distinct from author)")?;

    let self_assignment = match assign("auth") {
        Ok(()) => anyhow::bail!(
            "self-assignment (reviewer == target author) must be REJECTED — R22 distinct-from-author"
        ),
        Err(err) => err,
    };
    let gate_error = but_api::legacy::forge::classify_error(&self_assignment)
        .context("distinct-from-author rejection must surface a structured denial")?;
    assert_eq!(
        gate_error.code,
        but_authz::Denial::PERM_DENIED_CODE,
        "self-assignment must be perm.denied"
    );

    let rows = ctx
        .db
        .get_cache()?
        .local_review_assignments()
        .list_by_target(FEAT_REF)?;

    let rev2_rows: Vec<_> = rows
        .iter()
        .filter(|r| r.reviewer_principal == "rev2")
        .collect();
    assert_eq!(
        rev2_rows.len(),
        1,
        "second assign_reviewer(rev2) must UPDATE not duplicate — idempotent upsert"
    );
    assert_eq!(
        rev2_rows[0].state,
        but_authz::AssignmentState::Pending.name(),
        "rev2 assignment state must be pending"
    );

    assert!(
        rows.iter().any(|r| r.reviewer_principal == "rev3"),
        "rev3 assignment must be a distinct row"
    );
    assert!(
        !rows.iter().any(|r| r.reviewer_principal == "auth"),
        "self-assignment (auth) must NOT write a row — R22"
    );

    Ok(())
}

#[test]
#[serial_test::serial]
fn request_changes_review_implements_changes_requested_write() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_governed_repo();
    let ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();
    let runtime = tokio::runtime::Runtime::new()?;

    // Open the review and seed a pending assignment for `rev` so set_state has a
    // row to act on. `rev` itself holds reviews:write and is distinct from the
    // `auth` opener, so the assign succeeds.
    temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("auth")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || {
            runtime.block_on(but_api::legacy::forge::request_review(
                ctx.to_sync(),
                FEAT_REF.to_owned(),
                None,
            ))
        },
    )
    .context("opener must be established")?;

    temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("auth")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || {
            runtime.block_on(but_api::legacy::forge::assign_reviewer(
                ctx.to_sync(),
                FEAT_REF.to_owned(),
                "rev".to_owned(),
            ))
        },
    )
    .context("seed pending assignment for rev")?;

    let verdicts_before = ctx
        .db
        .get_cache()?
        .local_review_verdicts()
        .list_by_target(FEAT_REF)?
        .into_iter()
        .map(|v| (v.principal_id, v.verdict))
        .collect::<Vec<_>>();

    let result = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("rev")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || {
            runtime.block_on(but_api::legacy::forge::request_changes_review(
                ctx.to_sync(),
                FEAT_REF.to_owned(),
                Some("needs work".to_owned()),
            ))
        },
    );

    let err_text = match result {
        Ok(()) => String::new(),
        Err(err) => err.to_string(),
    };
    assert!(
        err_text.is_empty(),
        "request_changes_review must return Ok (the stub task_contract_invalid is gone); got: {err_text}"
    );
    assert!(
        !err_text.contains("task_contract_invalid") && !err_text.contains("no downstream"),
        "the stub blocker must NOT survive — got: {err_text}"
    );

    let rows = ctx
        .db
        .get_cache()?
        .local_review_assignments()
        .list_by_target(FEAT_REF)?;
    let rev_row = rows
        .iter()
        .find(|r| r.reviewer_principal == "rev")
        .context("rev assignment must still exist after the changes_requested flip")?;
    assert_eq!(
        rev_row.state,
        but_authz::AssignmentState::ChangesRequested.name(),
        "rev assignment state must be changes_requested after request_changes_review"
    );

    // Safe-seam proof: the assignment-state flip never reaches the merge gate,
    // because the gate reads only local_review_verdicts. Asserting the verdict
    // store is byte-identical before/after the flip is the local proof that the
    // merge-gate decision is unchanged.
    let verdicts_after = ctx
        .db
        .get_cache()?
        .local_review_verdicts()
        .list_by_target(FEAT_REF)?
        .into_iter()
        .map(|v| (v.principal_id, v.verdict))
        .collect::<Vec<_>>();
    assert_eq!(
        verdicts_before, verdicts_after,
        "merge-gate decision input (local_review_verdicts) must be unchanged by the changes_requested flip"
    );

    Ok(())
}

#[test]
#[serial_test::serial]
fn request_review_denied_without_authority_writes_nothing() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_governed_repo();
    let ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();
    let runtime = tokio::runtime::Runtime::new()?;

    let assignments_before = ctx
        .db
        .get_cache()?
        .local_review_assignments()
        .list_by_target(FEAT_REF)?;
    assert!(assignments_before.is_empty(), "fixture must seed empty");

    // `impl` holds contents:write ONLY — no pull_requests:write, no reviews:write.
    let request_err = match temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("impl")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || {
            runtime.block_on(but_api::legacy::forge::request_review(
                ctx.to_sync(),
                FEAT_REF.to_owned(),
                Some("rev2".to_owned()),
            ))
        },
    ) {
        Ok(()) => anyhow::bail!("impl lacks pull_requests:write; request_review must deny"),
        Err(err) => err,
    };
    let gate_error = but_api::legacy::forge::classify_error(&request_err)
        .context("request_review denial must be structured")?;
    assert_eq!(
        gate_error.code, "perm.denied",
        "request_review denial must be perm.denied"
    );
    assert!(
        gate_error.message.contains("pull_requests:write"),
        "request_review denial must name pull_requests:write, got: {}",
        gate_error.message
    );

    let changes_err = match temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("impl")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || {
            runtime.block_on(but_api::legacy::forge::request_changes_review(
                ctx.to_sync(),
                FEAT_REF.to_owned(),
                Some("fix".to_owned()),
            ))
        },
    ) {
        Ok(()) => anyhow::bail!("impl lacks reviews:write; request_changes_review must deny"),
        Err(err) => err,
    };
    let changes_gate = but_api::legacy::forge::classify_error(&changes_err)
        .context("request_changes_review denial must be structured")?;
    assert_eq!(changes_gate.code, "perm.denied");
    assert!(
        changes_gate.message.contains("reviews:write"),
        "request_changes_review denial must name reviews:write, got: {}",
        changes_gate.message
    );

    let assignments_after = ctx
        .db
        .get_cache()?
        .local_review_assignments()
        .list_by_target(FEAT_REF)?;
    let opener_after = ctx
        .db
        .get_cache()?
        .local_review_meta()
        .get(FEAT_REF, "opener_principal")?;
    assert!(
        assignments_after.is_empty(),
        "denied request_review must write NO assignment row (authorize-before-write)"
    );
    assert!(
        opener_after.is_none(),
        "denied request_review must write NO opener row"
    );

    Ok(())
}

#[test]
#[serial_test::serial]
fn request_review_is_local_cache_only_no_ref_mutation() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_governed_repo();
    let head_before = repo
        .find_reference(FEAT_REF)?
        .peel_to_commit()?
        .id
        .to_string();
    let refs_before = list_refs(&repo)?;
    let object_count_before = count_objects(&repo)?;
    let ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();
    let runtime = tokio::runtime::Runtime::new()?;

    temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("dev")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || {
            runtime.block_on(but_api::legacy::forge::request_review(
                ctx.to_sync(),
                FEAT_REF.to_owned(),
                Some("rev2".to_owned()),
            ))
        },
    )
    .context("request_review must succeed for the local-cache-only proof")?;

    // The assignment row MUST be written (local cache, like approve_review)…
    let assignments = ctx
        .db
        .get_cache()?
        .local_review_assignments()
        .list_by_target(FEAT_REF)?;
    assert_eq!(
        assignments.len(),
        1,
        "request_review must write the local-cache assignment row (no DryRun guard suppression)"
    );

    // …but no ref/object/oplog mutation occurs.
    let repo_after = ctx.repo.get()?;
    let head_after = repo_after
        .find_reference(FEAT_REF)?
        .peel_to_commit()?
        .id
        .to_string();
    let refs_after = list_refs(&repo_after)?;
    let object_count_after = count_objects(&repo_after)?;
    assert_eq!(
        head_before, head_after,
        "request_review must NOT move the branch HEAD"
    );
    assert_eq!(
        refs_before, refs_after,
        "request_review must NOT create or move any ref"
    );
    assert_eq!(
        object_count_before, object_count_after,
        "request_review must NOT write any new git object"
    );

    Ok(())
}

// ---- fixture + helpers -----------------------------------------------------

/// A governed repo for LPR proofs. Principals:
///   - `dev`    — pull_requests:write + contents:write (the open-PR caller)
///   - `rev`    — reviews:write (the assign/request-changes caller)
///   - `auth`   — pull_requests:write (the target author / opener)
///   - `impl`   — contents:write ONLY (the missing-authority negative control)
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

[[principal]]
id = "auth"
permissions = ["contents:write", "pull_requests:write", "reviews:write"]

[[principal]]
id = "impl"
permissions = ["contents:write"]
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

fn list_refs(repo: &gix::Repository) -> anyhow::Result<Vec<(String, String)>> {
    let mut refs = Vec::new();
    let it = repo.references()?;
    for item in it.all().map_err(|err| anyhow::anyhow!(err.to_string()))? {
        let item = item.map_err(|err| anyhow::anyhow!(err.to_string()))?;
        let name = item.name().as_bstr().to_string();
        let target = item
            .target()
            .try_id()
            .map(|id| id.to_string())
            .unwrap_or_default();
        refs.push((name, target));
    }
    refs.sort();
    Ok(refs)
}

fn count_objects(repo: &gix::Repository) -> anyhow::Result<usize> {
    // Count loose + packed objects via the object database. Local-cache-only
    // writes (SQLite rows) cannot reach this layer; any new assignment must
    // leave the count unchanged.
    let mut count = 0usize;
    let odb = repo.objects.clone();
    let iter = odb.iter().map_err(|err| anyhow::anyhow!(err.to_string()))?;
    for id in iter {
        if id.is_ok() {
            count += 1;
        }
    }
    Ok(count)
}
