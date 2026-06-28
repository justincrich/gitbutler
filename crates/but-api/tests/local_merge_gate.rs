const MAIN_REF: &str = "refs/heads/main";
const FEAT_REF: &str = "refs/heads/feat";

#[test]
#[serial_test::serial]
fn local_merge_gate_no_handle_denied() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let main_before = ref_id(&repo, MAIN_REF)?;
    let feat_before = ref_id(&repo, FEAT_REF)?;

    // No BUT_AGENT_HANDLE set → perm.denied
    temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", None),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || -> anyhow::Result<()> {
            let ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
            let err = assert_merge_gate_denied(
                but_api::legacy::merge_gate::enforce_local_merge_gate(&ctx, MAIN_REF, FEAT_REF),
                "perm.denied",
            );
            assert!(
                err.message.contains("BUT_AGENT_HANDLE") || err.message.contains("principal"),
                "perm.denied message should explain the missing handle"
            );
            assert_eq!(
                ref_id(&repo, MAIN_REF)?,
                main_before,
                "main ref must remain unchanged after no-handle denial"
            );
            assert_eq!(
                ref_id(&repo, FEAT_REF)?,
                feat_before,
                "feat ref must remain unchanged after no-handle denial"
            );
            Ok(())
        },
    )?;

    println!("no-handle local merge denied with `perm.denied`");
    Ok(())
}

#[test]
#[serial_test::serial]
fn local_merge_gate_unknown_handle_denied() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let main_before = ref_id(&repo, MAIN_REF)?;
    let feat_before = ref_id(&repo, FEAT_REF)?;

    temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("ghost")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || -> anyhow::Result<()> {
            let ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
            let err = assert_merge_gate_denied(
                but_api::legacy::merge_gate::enforce_local_merge_gate(&ctx, MAIN_REF, FEAT_REF),
                "perm.denied",
            );
            assert!(
                err.message.contains("ghost") || err.message.contains("unknown"),
                "perm.denied message should name the unknown handle"
            );
            assert_eq!(
                ref_id(&repo, MAIN_REF)?,
                main_before,
                "main ref must remain unchanged after unknown-handle denial"
            );
            assert_eq!(
                ref_id(&repo, FEAT_REF)?,
                feat_before,
                "feat ref must remain unchanged after unknown-handle denial"
            );
            Ok(())
        },
    )?;

    println!("unknown-handle local merge denied with `perm.denied`");
    Ok(())
}

#[test]
#[serial_test::serial]
fn local_merge_gate_readonly_denied() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let main_before = ref_id(&repo, MAIN_REF)?;
    let feat_before = ref_id(&repo, FEAT_REF)?;

    temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("ro")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || -> anyhow::Result<()> {
            let ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
            let err = assert_merge_gate_denied(
                but_api::legacy::merge_gate::enforce_local_merge_gate(&ctx, MAIN_REF, FEAT_REF),
                "perm.denied",
            );
            assert!(
                err.message.contains("merge") || err.message.contains("contents:write"),
                "perm.denied message should explain the missing merge authority"
            );
            assert_eq!(
                ref_id(&repo, MAIN_REF)?,
                main_before,
                "main ref must remain unchanged after readonly denial"
            );
            assert_eq!(
                ref_id(&repo, FEAT_REF)?,
                feat_before,
                "feat ref must remain unchanged after readonly denial"
            );
            Ok(())
        },
    )?;

    println!("readonly local merge denied with `perm.denied`");
    Ok(())
}

#[test]
#[serial_test::serial]
fn local_merge_gate_impl_no_merge_authority_denied() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let main_before = ref_id(&repo, MAIN_REF)?;
    let feat_before = ref_id(&repo, FEAT_REF)?;

    temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("impl")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || -> anyhow::Result<()> {
            let ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
            let err = assert_merge_gate_denied(
                but_api::legacy::merge_gate::enforce_local_merge_gate(&ctx, MAIN_REF, FEAT_REF),
                "perm.denied",
            );
            assert!(
                err.message.contains("merge") || err.message.contains("contents:write"),
                "perm.denied message should explain the missing merge authority"
            );
            assert_eq!(
                ref_id(&repo, MAIN_REF)?,
                main_before,
                "main ref must remain unchanged after no-merge-authority denial"
            );
            assert_eq!(
                ref_id(&repo, FEAT_REF)?,
                feat_before,
                "feat ref must remain unchanged after no-merge-authority denial"
            );
            Ok(())
        },
    )?;

    println!("impl (no merge authority) local merge denied with `perm.denied`");
    Ok(())
}

#[test]
#[serial_test::serial]
fn local_merge_gate_maint_no_approval_denied() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let main_before = ref_id(&repo, MAIN_REF)?;
    let feat_before = ref_id(&repo, FEAT_REF)?;

    temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("maint")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || -> anyhow::Result<()> {
            let ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
            let err = assert_merge_gate_denied(
                but_api::legacy::merge_gate::enforce_local_merge_gate(&ctx, MAIN_REF, FEAT_REF),
                "gate.review_required",
            );
            assert!(
                err.message.contains("review requirement") || err.message.contains("main"),
                "gate.review_required message should explain the missing approval"
            );
            assert!(
                err.unmet.iter().any(|u| u.contains("no_approval")),
                "unmet should contain no_approval when no approvals exist"
            );
            assert_eq!(
                ref_id(&repo, MAIN_REF)?,
                main_before,
                "main ref must remain unchanged after no-approval denial"
            );
            assert_eq!(
                ref_id(&repo, FEAT_REF)?,
                feat_before,
                "feat ref must remain unchanged after no-approval denial"
            );
            Ok(())
        },
    )?;

    println!("maint no-approval local merge denied with `gate.review_required`");
    Ok(())
}

#[test]
#[serial_test::serial]
fn local_merge_gate_maint_with_approval_allowed() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let feat_head = ref_id(&repo, FEAT_REF)?;

    temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("maint")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || -> anyhow::Result<()> {
            let ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();

            // Insert a head-pinned approval from a distinct reviewer
            ctx.db.get_cache_mut()?.local_review_verdicts_mut().insert(
                but_db::LocalReviewVerdict {
                    id: "test-approval-1".to_owned(),
                    target: "feat".to_owned(),
                    principal_id: "reviewer".to_owned(),
                    verdict: "approved".to_owned(),
                    head_oid: feat_head.to_string(),
                    created_at: chrono::Utc::now().naive_utc(),
                },
            )?;

            // Gate should allow the merge
            assert_no_governance_denial(
                but_api::legacy::merge_gate::enforce_local_merge_gate(&ctx, MAIN_REF, FEAT_REF),
                "maint with approval local merge",
            );
            Ok(())
        },
    )?;

    println!("maint with approval local merge allowed");
    Ok(())
}

#[test]
#[serial_test::serial]
fn local_merge_gate_ungoverned_repo_allowed() -> anyhow::Result<()> {
    let (repo, _tmp) = repo_with_no_governance_config();
    let _main_before = ref_id(&repo, MAIN_REF)?;
    let _feat_before = ref_id(&repo, FEAT_REF)?;

    // Ungoverned repo: no BUT_AGENT_HANDLE should still allow (gate short-circuits)
    temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", None),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || -> anyhow::Result<()> {
            let ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
            assert_no_governance_denial(
                but_api::legacy::merge_gate::enforce_local_merge_gate(&ctx, MAIN_REF, FEAT_REF),
                "ungoverned repo local merge",
            );
            assert_eq!(
                ref_id(&repo, MAIN_REF)?,
                _main_before,
                "ungoverned main ref should remain unchanged"
            );
            assert_eq!(
                ref_id(&repo, FEAT_REF)?,
                _feat_before,
                "ungoverned feat ref should remain unchanged"
            );
            Ok(())
        },
    )?;

    println!("ungoverned repo local merge allowed (opt-in by presence)");
    Ok(())
}

/// Fixture: governed repo with main (protected) and feat (unprotected).
/// Uses `permissions.toml` with `[[principal]]` blocks (NOT `agents.toml`).
/// Sets up review gate on main requiring 1 approval distinct from author.
fn governed_repo() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "ro"
permissions = ["contents:read"]

[[principal]]
id = "impl"
permissions = ["contents:write"]

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

[[branch]]
name = "feat"
protected = false

[[gate]]
branch = "main"
type = "review"
min_approvals = 1
require_distinct_from_author = true
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

/// Fixture: repo with NO committed governance config (ungoverned case).
fn repo_with_no_governance_config() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
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

fn ref_id(repo: &gix::Repository, ref_name: &str) -> anyhow::Result<gix::ObjectId> {
    Ok(repo.find_reference(ref_name)?.peel_to_id()?.detach())
}

fn assert_merge_gate_denied(
    result: anyhow::Result<()>,
    code: &'static str,
) -> but_api::legacy::merge_gate::MergeGateError {
    match result {
        Ok(_) => panic!("local merge gate should deny with {code}"),
        Err(err) => {
            let gate_error = but_api::legacy::merge_gate::classify_error(&err)
                .expect("local merge gate errors should be structured");
            assert_eq!(
                gate_error.code, code,
                "local merge gate should return the expected stable code"
            );
            gate_error
        }
    }
}

fn assert_no_governance_denial(result: anyhow::Result<()>, label: &str) {
    if let Err(err) = result {
        assert!(
            but_api::legacy::merge_gate::classify_error(&err).is_none(),
            "{label} should not be rejected by the local merge gate, got: {err:#}"
        );
    }
}
