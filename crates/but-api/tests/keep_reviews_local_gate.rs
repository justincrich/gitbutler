//! LPR-006 integration proof for the `keep_reviews_local` operator-preference
//! gate on `request_review` (AC-3): when `keep_reviews_local == false`, the
//! remote-mirror path is a NAMED SEAM ONLY — `request_review` returns a
//! structured error saying the remote mirror is not yet implemented, and never
//! reaches the local-cache writes (the mirror is gated, not built).
//!
//! Reuses the same governed-repo fixture as `local_review_assignments.rs`:
//! real but-db + real gix, no mocks.

use anyhow::Context as _;

const FEAT_REF: &str = "refs/heads/feat";

#[test]
#[serial_test::serial]
fn keep_reviews_local_false_returns_named_seam_error() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_governed_repo();
    let mut ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();

    // LPR-006 R21: the operator preference is NOT admin-gated, NOT ref-pinned.
    // An operator who flips `keep_reviews_local = false` is asking for the
    // remote-mirror path — which is a NAMED SEAM ONLY in LPR scope (no mirror
    // code runs). The gate must surface a structured error so callers can
    // distinguish "preference says mirror" from "mirror not yet built".
    *ctx.legacy_project.keep_reviews_local = false;

    let runtime = tokio::runtime::Runtime::new()?;
    let err = match temp_env::with_vars(
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
    ) {
        Ok(()) => anyhow::bail!(
            "keep_reviews_local=false must surface the named seam error, not succeed silently"
        ),
        Err(err) => err,
    };

    // The error must be structured (consumer-facing code + message), not a
    // bare anyhow string — `classify_error` is the API-boundary surface for
    // forge-gate errors.
    let gate_error = but_api::legacy::forge::classify_error(&err)
        .context("the named-seam error must be structured (consumable via classify_error)")?;

    assert_eq!(
        gate_error.code,
        but_api::legacy::forge::REMOTE_MIRROR_NOT_IMPLEMENTED_CODE,
        "the named-seam error code must be the stable remote-mirror code"
    );
    assert!(
        gate_error
            .message
            .contains("remote review mirror not yet implemented"),
        "the named-seam error message must name the missing remote mirror; got: {}",
        gate_error.message
    );

    // Authorize-before-write discipline: the gate fires BEFORE any local-cache
    // write, so a `keep_reviews_local=false` call must NOT have produced
    // assignment / opener rows.
    let db = ctx.db.get_cache()?;
    let assignments = db.local_review_assignments().list_by_target(FEAT_REF)?;
    let opener = db.local_review_meta().get(FEAT_REF, "opener_principal")?;
    assert!(
        assignments.is_empty(),
        "the named-seam gate must fire before the assignment write — no rows may exist"
    );
    assert!(
        opener.is_none(),
        "the named-seam gate must fire before the opener write — no opener row may exist"
    );

    Ok(())
}

// ---- fixture (mirrors local_review_assignments::lpr_governed_repo) -----------

/// A governed repo for the LPR-006 keep_reviews_local gate proof. Principals:
///   - `dev` — pull_requests:write + contents:write (the open-PR caller)
fn lpr_governed_repo() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "dev"
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
