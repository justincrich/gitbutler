use but_core::{DiffSpec, DryRun};
use but_rebase::graph_rebase::mutate::{InsertSide, RelativeTo};

const MAIN_REF: &str = "refs/heads/main";
const FEAT_REF: &str = "refs/heads/feat";

#[test]
#[serial_test::serial]
fn commit_gate_feature_ok_protected_rejected() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let main_before = ref_id(&repo, MAIN_REF)?;
    let feat_before = ref_id(&repo, FEAT_REF)?;

    temp_env::with_var("BUT_AGENT_HANDLE", Some("dev"), || -> anyhow::Result<()> {
        checkout(&repo, "feat");
        write_file(&repo, "feature.txt", "feature\n")?;
        let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
        let feature = commit_to_ref(&mut ctx, FEAT_REF, "feature commit", DryRun::No)?;
        assert!(
            feature.new_commit.is_some(),
            "contents:write principal should create a feature commit"
        );
        assert_ne!(
            ref_id(&repo, FEAT_REF)?,
            feat_before,
            "feature ref should advance after an allowed commit"
        );

        checkout(&repo, "main");
        write_file(&repo, "main.txt", "main\n")?;
        let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
        let err = match commit_to_ref(&mut ctx, MAIN_REF, "direct main commit", DryRun::No) {
            Ok(_) => anyhow::bail!("protected main direct commit should be denied"),
            Err(err) => err,
        };
        let denial = err
            .downcast_ref::<but_authz::Denial>()
            .expect("protected branch rejection should be a structured authz denial");
        assert_eq!(
            denial.code, "branch.protected",
            "protected branch denial must use the stable branch.protected code"
        );
        assert!(
            denial.message.contains("main"),
            "branch.protected message should name the rejected main branch"
        );
        assert_eq!(
            ref_id(&repo, MAIN_REF)?,
            main_before,
            "main ref must remain unchanged after protected branch denial"
        );
        println!("`feat` HEAD sha advanced from {feat_before}");
        println!("`error.code == \"branch.protected\"` and message names `main`");
        println!("`main` HEAD sha == seeded base sha");
        Ok(())
    })?;

    Ok(())
}

#[test]
#[serial_test::serial]
fn commit_gate_readonly_and_bad_handle_denied() -> anyhow::Result<()> {
    for (handle, label) in [
        (Some("ro"), "read-only"),
        (None, "unset"),
        (Some(""), "empty"),
        (Some("ghost"), "ghost"),
    ] {
        let (repo, _tmp) = governed_repo();
        let feat_before = ref_id(&repo, FEAT_REF)?;

        temp_env::with_var("BUT_AGENT_HANDLE", handle, || -> anyhow::Result<()> {
            checkout(&repo, "feat");
            write_file(&repo, &format!("{label}.txt"), label)?;
            let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
            let err = assert_commit_denied(
                commit_to_ref(&mut ctx, FEAT_REF, &format!("{label} commit"), DryRun::No),
                "perm.denied",
            );
            assert!(
                err.message.contains("contents:write")
                    || err.message.contains("BUT_AGENT_HANDLE")
                    || err.message.contains("ghost"),
                "perm.denied message should explain the rejected handle or missing contents:write"
            );
            assert_eq!(
                ref_id(&repo, FEAT_REF)?,
                feat_before,
                "{label} denial must leave feat unchanged"
            );
            println!("`{label}` commit denied with `error.code == \"perm.denied\"`");
            Ok(())
        })?;
    }

    Ok(())
}

#[test]
#[serial_test::serial]
fn commit_gate_edit_cannot_unprotect() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let main_before = ref_id(&repo, MAIN_REF)?;

    temp_env::with_var("BUT_AGENT_HANDLE", Some("dev"), || -> anyhow::Result<()> {
        checkout(&repo, "main");
        but_testsupport::invoke_bash(
            r#"
cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = false
EOF
"#,
            &repo,
        );
        write_file(&repo, "working-tree-unprotect.txt", "still denied\n")?;
        let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
        assert_commit_denied(
            commit_to_ref(&mut ctx, MAIN_REF, "working tree unprotect", DryRun::No),
            "branch.protected",
        );
        assert_eq!(
            ref_id(&repo, MAIN_REF)?,
            main_before,
            "uncommitted gates.toml edits must not unprotect main"
        );

        reset_worktree(&repo);
        checkout(&repo, "feat");
        but_testsupport::invoke_bash(
            r#"
cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = false
EOF
git add .gitbutler/gates.toml
git commit -m "feature unprotects main"
"#,
            &repo,
        );
        checkout(&repo, "main");
        write_file(&repo, "feature-head-unprotect.txt", "still denied\n")?;
        let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
        assert_commit_denied(
            commit_to_ref(&mut ctx, MAIN_REF, "feature head unprotect", DryRun::No),
            "branch.protected",
        );
        assert_eq!(
            ref_id(&repo, MAIN_REF)?,
            main_before,
            "feature-head gates.toml must not unprotect target ref main"
        );
        println!("target-ref `main` gates.toml controls protection in both unprotect attempts");
        Ok(())
    })?;

    Ok(())
}

#[test]
#[serial_test::serial]
fn commit_gate_malformed_partial_and_dryrun() -> anyhow::Result<()> {
    temp_env::with_var("BUT_AGENT_HANDLE", Some("dev"), || -> anyhow::Result<()> {
        // Malformed governance file at the target ref -> config.invalid (fail closed).
        let (repo, _tmp) = governed_repo();
        checkout(&repo, "feat");
        but_testsupport::invoke_bash(
            r#"
cat >.gitbutler/gates.toml <<'EOF'
[[branch]
name = "feat"
protected = nope
EOF
git add .gitbutler/gates.toml
git commit -m "malformed feat gates"
"#,
            &repo,
        );
        write_file(&repo, "malformed.txt", "malformed\n")?;
        let feat_before = ref_id(&repo, FEAT_REF)?;
        let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
        assert_commit_denied(
            commit_to_ref(&mut ctx, FEAT_REF, "malformed config", DryRun::No),
            "config.invalid",
        );
        assert_eq!(ref_id(&repo, FEAT_REF)?, feat_before);

        // PARTIAL (incomplete) governance: governance is opted-in (permissions.toml
        // committed) but the companion gates.toml is missing -> config.invalid (fail
        // closed on incomplete governance). This is DISTINCT from the fully-absent
        // case (zero files), which is ungoverned and allowed -- see
        // `commit_gate_absent_config_is_ungoverned`.
        let (repo, _tmp) = repo_with_partial_governance_config();
        checkout(&repo, "feat");
        write_file(&repo, "partial.txt", "partial\n")?;
        let feat_before = ref_id(&repo, FEAT_REF)?;
        let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
        assert_commit_denied(
            commit_to_ref(&mut ctx, FEAT_REF, "partial config", DryRun::No),
            "config.invalid",
        );
        assert_eq!(ref_id(&repo, FEAT_REF)?, feat_before);

        // DryRun does not bypass the gate: a denied protected-branch commit still
        // denies and persists nothing.
        let (repo, _tmp) = governed_repo();
        checkout(&repo, "main");
        write_file(&repo, "denied-dryrun.txt", "denied dry run\n")?;
        let main_before = ref_id(&repo, MAIN_REF)?;
        let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
        assert_commit_denied(
            commit_to_ref(&mut ctx, MAIN_REF, "denied dry run", DryRun::Yes),
            "branch.protected",
        );
        assert_eq!(
            ref_id(&repo, MAIN_REF)?,
            main_before,
            "denied dry run must leave main unchanged"
        );

        // An allowed DryRun previews a commit object but persists nothing: it MUST
        // still produce a preview commit, and that object MUST be absent from the
        // live odb (strong assertion -- not vacuously skipped when new_commit None).
        reset_worktree(&repo);
        checkout(&repo, "feat");
        write_file(&repo, "allowed-dryrun.txt", "allowed dry run\n")?;
        let feat_before = ref_id(&repo, FEAT_REF)?;
        let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
        let outcome = commit_to_ref(&mut ctx, FEAT_REF, "allowed dry run", DryRun::Yes)?;
        assert_eq!(
            ref_id(&repo, FEAT_REF)?,
            feat_before,
            "allowed dry run must preview without advancing feat"
        );
        let preview_commit = outcome
            .new_commit
            .expect("an allowed DryRun must still preview a commit object");
        assert!(
            repo.find_object(preview_commit).is_err(),
            "allowed dry run commit object must not be persisted in the live repo"
        );
        println!("malformed and partial (incomplete) config return `config.invalid`");
        println!("denied and allowed DryRun commits persist no ref/object");
        Ok(())
    })?;

    Ok(())
}

#[test]
#[serial_test::serial]
fn commit_gate_absent_config_is_ungoverned() -> anyhow::Result<()> {
    // Governance is OPT-IN BY PRESENCE (RF-010 amended): a target ref with NO
    // committed `.gitbutler/*.toml` is ungoverned -- the gate does not run and the
    // commit is allowed. Under a governed repo the same inputs would be denied
    // (an unset handle -> perm.denied; an `ro` handle lacking contents:write ->
    // perm.denied), so a successful commit here proves the gate was skipped, not
    // that authorization passed. Landing on a *governed* trunk is still mediated by
    // the merge gate (Sprint 01b), so opt-in does not weaken a governed branch.
    for (handle, label) in [(None, "unset"), (Some("ro"), "read-only")] {
        let (repo, _tmp) = repo_with_no_governance_config();
        let feat_before = ref_id(&repo, FEAT_REF)?;
        temp_env::with_var("BUT_AGENT_HANDLE", handle, || -> anyhow::Result<()> {
            checkout(&repo, "feat");
            write_file(&repo, &format!("ungoverned-{label}.txt"), label)?;
            let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
            let outcome = commit_to_ref(
                &mut ctx,
                FEAT_REF,
                &format!("ungoverned {label} commit"),
                DryRun::No,
            )?;
            assert!(
                outcome.new_commit.is_some(),
                "ungoverned repo ({label} handle) must allow a commit -- no governance config is committed"
            );
            assert_ne!(
                ref_id(&repo, FEAT_REF)?,
                feat_before,
                "feat ref should advance in an ungoverned repo ({label} handle)"
            );
            Ok(())
        })?;
    }
    println!(
        "a target ref with no committed governance config is ungoverned: commits are allowed (opt-in by presence)"
    );

    Ok(())
}

/// Anti-fakeability harness: the three governance fixtures MUST be structurally
/// distinct so a future edit cannot silently substitute a partial config for the
/// truly-absent case (the exact defect this remediation closed). A no-config repo
/// commits ZERO governance files, a partial repo exactly ONE, a governed repo BOTH.
#[test]
fn governance_fixtures_are_structurally_distinct() -> anyhow::Result<()> {
    let (no_config, _t1) = repo_with_no_governance_config();
    let (partial, _t2) = repo_with_partial_governance_config();
    let (governed, _t3) = governed_repo();

    assert_eq!(
        governance_file_count(&no_config, MAIN_REF)?,
        0,
        "the no-config fixture must commit ZERO governance files (a true absent case, not a partial one)"
    );
    assert_eq!(
        governance_file_count(&partial, MAIN_REF)?,
        1,
        "the partial fixture must commit EXACTLY ONE governance file (incomplete governance)"
    );
    assert_eq!(
        governance_file_count(&governed, MAIN_REF)?,
        2,
        "the governed fixture must commit BOTH governance files"
    );

    Ok(())
}

#[test]
#[serial_test::serial]
fn commit_gate_commit_relative_checks_contents_write_without_branch_protection()
-> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let main_before = ref_id(&repo, MAIN_REF)?;

    temp_env::with_var("BUT_AGENT_HANDLE", Some("ro"), || -> anyhow::Result<()> {
        checkout(&repo, "main");
        write_file(&repo, "commit-relative-ro.txt", "readonly\n")?;
        let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
        let err = assert_commit_denied(
            commit_to_commit(
                &mut ctx,
                main_before,
                "readonly commit-relative",
                DryRun::No,
            ),
            "perm.denied",
        );
        assert!(
            err.message.contains("contents:write"),
            "commit-relative denial should require contents:write"
        );
        assert_eq!(
            ref_id(&repo, MAIN_REF)?,
            main_before,
            "readonly commit-relative denial must leave protected main unchanged"
        );
        Ok(())
    })?;

    reset_worktree(&repo);

    temp_env::with_var("BUT_AGENT_HANDLE", Some("dev"), || -> anyhow::Result<()> {
        checkout(&repo, "main");
        write_file(&repo, "commit-relative-dev.txt", "dev\n")?;
        let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
        let outcome = commit_to_commit(&mut ctx, main_before, "dev commit-relative", DryRun::No)?;
        assert!(
            outcome.new_commit.is_some(),
            "contents:write principal should create a commit relative to a protected branch commit"
        );
        Ok(())
    })?;

    Ok(())
}

#[test]
#[serial_test::serial]
fn commit_gate_worktree_integrate_protected_rejected() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let main_before = ref_id(&repo, MAIN_REF)?;
    let feature_target = gix::refs::FullName::try_from(FEAT_REF)?;
    let protected_target = gix::refs::FullName::try_from(MAIN_REF)?;

    temp_env::with_var("BUT_AGENT_HANDLE", Some("dev"), || -> anyhow::Result<()> {
        let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
        let worktree_id = but_worktrees::WorktreeId::generate();
        let protected_result =
            but_api::legacy::worktree::worktree_integrate(&mut ctx, worktree_id, protected_target);
        let denial = assert_governance_denied(protected_result, "branch.protected");
        assert!(
            denial.message.contains("main"),
            "branch.protected message should name the protected main branch"
        );
        assert_eq!(
            ref_id(&repo, MAIN_REF)?,
            main_before,
            "main ref must remain unchanged after protected worktree integration denial"
        );

        let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
        let worktree_id = but_worktrees::WorktreeId::generate();
        let feature_result =
            but_api::legacy::worktree::worktree_integrate(&mut ctx, worktree_id, feature_target);
        assert_no_governance_denial(feature_result, "feature-target worktree integration");

        Ok(())
    })?;

    Ok(())
}

#[test]
#[serial_test::serial]
fn commit_gate_apply_integrate_readonly_denied() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let feat = gix::refs::FullName::try_from(FEAT_REF)?;
    let feat_before = ref_id(&repo, FEAT_REF)?;

    {
        let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
        set_default_target_to_origin_main(&mut ctx, &repo)?;
    }

    temp_env::with_var("BUT_AGENT_HANDLE", Some("ro"), || -> anyhow::Result<()> {
        let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
        let apply_denial = assert_governance_denied(
            but_api::branch::apply(&mut ctx, feat.as_ref()),
            "perm.denied",
        );
        assert!(
            apply_denial.message.contains("contents:write"),
            "branch::apply denial should name the missing contents:write permission"
        );
        assert_eq!(
            ref_id(&repo, FEAT_REF)?,
            feat_before,
            "readonly branch::apply denial must leave feat unchanged"
        );

        let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
        let integration = integration_plan_for_branch(&ctx, feat.as_ref())?;
        let integrate_denial = assert_governance_denied(
            but_api::branch::apply_branch_integration(
                &mut ctx,
                feat.as_ref(),
                integration,
                DryRun::No,
            ),
            "perm.denied",
        );
        assert!(
            integrate_denial.message.contains("contents:write"),
            "apply_branch_integration denial should name the missing contents:write permission"
        );
        assert_eq!(
            ref_id(&repo, FEAT_REF)?,
            feat_before,
            "readonly apply_branch_integration denial must leave feat unchanged"
        );

        Ok(())
    })?;

    reset_worktree(&repo);

    temp_env::with_var("BUT_AGENT_HANDLE", Some("dev"), || -> anyhow::Result<()> {
        let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
        assert_no_governance_denial(
            but_api::branch::apply(&mut ctx, feat.as_ref()),
            "contents:write branch::apply",
        );

        let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
        let integration = integration_plan_for_branch(&ctx, feat.as_ref())?;
        assert_no_governance_denial(
            but_api::branch::apply_branch_integration(
                &mut ctx,
                feat.as_ref(),
                integration,
                DryRun::No,
            ),
            "contents:write apply_branch_integration",
        );

        Ok(())
    })?;

    Ok(())
}

#[test]
#[serial_test::serial]
fn commit_gate_apply_integrate_no_target_ungoverned() -> anyhow::Result<()> {
    let (repo, _tmp) = repo_with_no_governance_config();
    let feat = gix::refs::FullName::try_from(FEAT_REF)?;

    temp_env::with_var("BUT_AGENT_HANDLE", Some("ro"), || -> anyhow::Result<()> {
        let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
        clear_default_target(&mut ctx)?;
        assert_no_governance_denial(
            but_api::branch::apply(&mut ctx, feat.as_ref()),
            "no-target branch::apply",
        );

        let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
        clear_default_target(&mut ctx)?;
        let integration = empty_body_integration_plan(&repo)?;
        assert_no_governance_denial(
            but_api::branch::apply_branch_integration(
                &mut ctx,
                feat.as_ref(),
                integration,
                DryRun::No,
            ),
            "no-target apply_branch_integration",
        );

        Ok(())
    })?;

    Ok(())
}

#[test]
#[serial_test::serial]
fn commit_gate_governed_missing_target_failclosed() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let feat = gix::refs::FullName::try_from(FEAT_REF)?;
    let feat_before = ref_id(&repo, FEAT_REF)?;

    assert_eq!(
        governance_file_count(&repo, MAIN_REF)?,
        2,
        "the governed missing-target fixture must commit governance on main"
    );

    temp_env::with_var("BUT_AGENT_HANDLE", Some("ro"), || -> anyhow::Result<()> {
        let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
        clear_default_target(&mut ctx)?;
        let apply_denial = assert_governance_denied(
            but_api::branch::apply(&mut ctx, feat.as_ref()),
            "perm.denied",
        );
        assert!(
            apply_denial.message.contains("contents:write"),
            "governed no-target branch::apply must fail closed through contents:write"
        );
        assert_eq!(
            ref_id(&repo, FEAT_REF)?,
            feat_before,
            "governed no-target branch::apply denial must leave feat unchanged"
        );

        let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
        clear_default_target(&mut ctx)?;
        let integration = empty_body_integration_plan(&repo)?;
        let integrate_denial = assert_governance_denied(
            but_api::branch::apply_branch_integration(
                &mut ctx,
                feat.as_ref(),
                integration,
                DryRun::No,
            ),
            "perm.denied",
        );
        assert!(
            integrate_denial.message.contains("contents:write"),
            "governed no-target apply_branch_integration must fail closed through contents:write"
        );
        assert_eq!(
            ref_id(&repo, FEAT_REF)?,
            feat_before,
            "governed no-target apply_branch_integration denial must leave feat unchanged"
        );

        Ok(())
    })?;

    Ok(())
}

#[test]
#[serial_test::serial]
fn commit_gate_apply_integrate_dryrun_targetref_pinned() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let feat = gix::refs::FullName::try_from(FEAT_REF)?;
    let feat_before = ref_id(&repo, FEAT_REF)?;

    {
        let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
        set_default_target_to_origin_main(&mut ctx, &repo)?;
    }

    temp_env::with_var("BUT_AGENT_HANDLE", Some("ro"), || -> anyhow::Result<()> {
        let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
        let integration = integration_plan_for_branch(&ctx, feat.as_ref())?;
        let dryrun_denial = assert_governance_denied(
            but_api::branch::apply_branch_integration(
                &mut ctx,
                feat.as_ref(),
                integration,
                DryRun::Yes,
            ),
            "perm.denied",
        );
        assert!(
            dryrun_denial.message.contains("contents:write"),
            "DryRun integrate denial should name the missing contents:write permission"
        );
        assert_eq!(
            ref_id(&repo, FEAT_REF)?,
            feat_before,
            "denied DryRun apply_branch_integration must leave feat unchanged"
        );

        reset_worktree(&repo);
        weaken_worktree_governance(&repo);
        let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
        let apply_denial = assert_governance_denied(
            but_api::branch::apply(&mut ctx, feat.as_ref()),
            "perm.denied",
        );
        assert!(
            apply_denial.message.contains("contents:write"),
            "working-tree governance edits must not grant contents:write"
        );
        assert_eq!(
            ref_id(&repo, FEAT_REF)?,
            feat_before,
            "working-tree governance edit must not let readonly apply advance feat"
        );

        Ok(())
    })?;

    assert_gate_helper_call_count("src/branch.rs", 2)?;
    assert_gate_helper_call_count("src/legacy/worktree.rs", 1)?;

    Ok(())
}

#[test]
fn commit_gate_generated_entrypoint_authorizes_before_exclusive_guard() -> anyhow::Result<()> {
    let source =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/commit/create.rs"))?;
    let annotated_start = source.find("#[but_api(napi").ok_or_else(|| {
        anyhow::anyhow!("commit_create must remain the generated N-API entrypoint")
    })?;
    let signature_end = source[annotated_start..]
        .find(") -> anyhow::Result<CommitCreateResult>")
        .ok_or_else(|| {
            anyhow::anyhow!("commit_create signature should return CommitCreateResult")
        })?;
    let signature = &source[annotated_start..annotated_start + signature_end];

    assert!(
        !signature.contains("RepoExclusive"),
        "annotated commit_create must not take RepoExclusive because the macro acquires it before the function body"
    );
    assert!(
        source.contains("fn commit_create_with_perm("),
        "commit_create should delegate to a permission-taking implementation after authorization"
    );

    Ok(())
}

fn governed_repo() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "dev"
permissions = ["contents:write"]

[[principal]]
id = "ro"
permissions = ["contents:read"]
EOF

cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true

[[branch]]
name = "feat"
protected = false
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

fn repo_with_partial_governance_config() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "dev"
permissions = ["contents:write"]
EOF
git add .gitbutler/permissions.toml
git commit -m "partial governance config"
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

/// A repo with NO committed `.gitbutler/*.toml` governance config at any target
/// ref -- the truly-absent (ungoverned) case. Commits the same `feat` branch as
/// the governed fixtures so `commit_to_ref(FEAT_REF, ..)` is comparable, but
/// commits zero governance files.
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

fn set_default_target_to_origin_main(
    ctx: &mut but_ctx::Context,
    repo: &gix::Repository,
) -> anyhow::Result<()> {
    but_testsupport::invoke_bash(
        "git update-ref refs/remotes/origin/main refs/heads/main",
        repo,
    );
    let target = gix::refs::FullName::try_from("refs/remotes/origin/main")?;
    but_api::branch::set_default_target(ctx, target.as_ref(), Some("origin".to_owned()))?;
    Ok(())
}

fn clear_default_target(ctx: &mut but_ctx::Context) -> anyhow::Result<()> {
    let mut project_meta = ctx.project_meta()?;
    project_meta.target_ref = None;
    project_meta.target_commit_id = None;
    ctx.set_project_meta(project_meta)?;
    Ok(())
}

fn integration_plan_for_branch(
    ctx: &but_ctx::Context,
    branch: &gix::refs::FullNameRef,
) -> anyhow::Result<but_api::branch::json::InteractiveIntegration> {
    let initial = but_api::branch::get_initial_branch_integration(
        ctx,
        branch,
        Some(but_api::branch::json::BranchIntegrationStrategy::PullRebase),
    );
    match initial {
        Ok(initial) => Ok(workspace_integration_to_json(initial.integration)),
        Err(_) => {
            let repo = ctx.repo.get()?;
            empty_body_integration_plan(&repo)
        }
    }
}

fn empty_body_integration_plan(
    repo: &gix::Repository,
) -> anyhow::Result<but_api::branch::json::InteractiveIntegration> {
    let merge_base = ref_id(repo, MAIN_REF)?;
    Ok(but_api::branch::json::InteractiveIntegration {
        merge_base: merge_base.into(),
        first_local_not_integrated: None,
        steps: vec![but_api::branch::json::InteractiveIntegrationStep::Pick {
            commit_id: ref_id(repo, FEAT_REF)?.into(),
        }],
    })
}

fn workspace_integration_to_json(
    integration: but_workspace::branch::integrate_branch_upstream::InteractiveIntegration,
) -> but_api::branch::json::InteractiveIntegration {
    but_api::branch::json::InteractiveIntegration {
        merge_base: integration.merge_base.into(),
        first_local_not_integrated: integration.first_local_not_integrated.map(Into::into),
        steps: integration
            .steps
            .into_iter()
            .map(|step| match step {
                but_workspace::branch::InteractiveIntegrationStep::Pick { commit_id } => {
                    but_api::branch::json::InteractiveIntegrationStep::Pick {
                        commit_id: commit_id.into(),
                    }
                }
                but_workspace::branch::InteractiveIntegrationStep::Squash { commits, message } => {
                    but_api::branch::json::InteractiveIntegrationStep::Squash {
                        commits: commits.into_iter().map(Into::into).collect(),
                        message,
                    }
                }
                but_workspace::branch::InteractiveIntegrationStep::Merge { commit_id } => {
                    but_api::branch::json::InteractiveIntegrationStep::Merge {
                        commit_id: commit_id.into(),
                    }
                }
            })
            .collect(),
    }
}

fn weaken_worktree_governance(repo: &gix::Repository) {
    but_testsupport::invoke_bash(
        r#"
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "ro"
permissions = ["contents:write"]
EOF

cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = false

[[branch]]
name = "feat"
protected = false
EOF
"#,
        repo,
    );
}

fn assert_gate_helper_call_count(
    relative_path: &str,
    expected_minimum: usize,
) -> anyhow::Result<()> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path),
    )?;
    let count = source.matches("enforce_commit_gate_for_target").count();
    assert!(
        count >= expected_minimum,
        "{relative_path} should call enforce_commit_gate_for_target at least {expected_minimum} time(s), found {count}"
    );
    Ok(())
}

/// Count how many of the two governance files are committed in `target_ref`'s
/// tree. Used by the anti-fakeability harness to keep the fixtures distinct.
fn governance_file_count(repo: &gix::Repository, target_ref: &str) -> anyhow::Result<usize> {
    let tree = repo.find_reference(target_ref)?.peel_to_commit()?.tree()?;
    let mut count = 0;
    for path in [".gitbutler/permissions.toml", ".gitbutler/gates.toml"] {
        if tree
            .lookup_entry_by_path(std::path::Path::new(path))?
            .is_some()
        {
            count += 1;
        }
    }
    Ok(count)
}

fn commit_to_ref(
    ctx: &mut but_ctx::Context,
    ref_name: &str,
    message: &str,
    dry_run: DryRun,
) -> anyhow::Result<but_api::commit::types::CommitCreateResult> {
    let repo = ctx.repo.get()?.clone();
    let changes = worktree_changes_as_specs(&repo)?;
    but_api::commit::create::commit_create_only(
        ctx,
        RelativeTo::Reference(gix::refs::FullName::try_from(ref_name)?),
        InsertSide::Below,
        changes,
        message.to_owned(),
        dry_run,
    )
}

fn commit_to_commit(
    ctx: &mut but_ctx::Context,
    commit_id: gix::ObjectId,
    message: &str,
    dry_run: DryRun,
) -> anyhow::Result<but_api::commit::types::CommitCreateResult> {
    let repo = ctx.repo.get()?.clone();
    let changes = worktree_changes_as_specs(&repo)?;
    but_api::commit::create::commit_create_only(
        ctx,
        RelativeTo::Commit(commit_id),
        InsertSide::Above,
        changes,
        message.to_owned(),
        dry_run,
    )
}

fn worktree_changes_as_specs(repo: &gix::Repository) -> anyhow::Result<Vec<DiffSpec>> {
    Ok(but_core::diff::worktree_changes(repo)?
        .changes
        .into_iter()
        .map(DiffSpec::from)
        .collect())
}

fn write_file(repo: &gix::Repository, path: &str, content: &str) -> anyhow::Result<()> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow::anyhow!("test repository must be non-bare"))?;
    std::fs::write(workdir.join(path), content)?;
    Ok(())
}

fn checkout(repo: &gix::Repository, branch_name: &str) {
    but_testsupport::invoke_bash(&format!("git checkout {branch_name}"), repo);
}

fn reset_worktree(repo: &gix::Repository) {
    but_testsupport::invoke_bash("git reset --hard && git clean -fd", repo);
}

fn ref_id(repo: &gix::Repository, ref_name: &str) -> anyhow::Result<gix::ObjectId> {
    Ok(repo.find_reference(ref_name)?.peel_to_id()?.detach())
}

fn assert_commit_denied(
    result: anyhow::Result<but_api::commit::types::CommitCreateResult>,
    code: &'static str,
) -> but_api::commit::create::gate::CommitGateError {
    match result {
        Ok(_) => panic!("commit should be denied with {code}"),
        Err(err) => {
            let gate_error = but_api::commit::create::gate::classify_error(&err)
                .expect("commit gate errors should be structured");
            assert_eq!(
                gate_error.code, code,
                "commit gate should return the expected stable code"
            );
            gate_error
        }
    }
}

fn assert_governance_denied<T>(
    result: anyhow::Result<T>,
    code: &'static str,
) -> but_api::commit::create::gate::CommitGateError {
    match result {
        Ok(_) => panic!("operation should be denied by the commit gate with {code}"),
        Err(err) => {
            let gate_error =
                but_api::commit::create::gate::classify_error(&err).unwrap_or_else(|| {
                    panic!(
                        "operation should fail with a structured commit gate denial, got: {err:#}"
                    )
                });
            assert_eq!(
                gate_error.code, code,
                "commit gate should return the expected stable code"
            );
            gate_error
        }
    }
}

fn assert_no_governance_denial<T>(result: anyhow::Result<T>, label: &str) {
    if let Err(err) = result {
        assert!(
            but_api::commit::create::gate::classify_error(&err).is_none(),
            "{label} should not be rejected by the commit gate, got: {err:#}"
        );
    }
}
