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
fn commit_gate_malformed_absent_and_dryrun() -> anyhow::Result<()> {
    temp_env::with_var("BUT_AGENT_HANDLE", Some("dev"), || -> anyhow::Result<()> {
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

        let (repo, _tmp) = repo_with_partial_governance_config();
        checkout(&repo, "feat");
        write_file(&repo, "absent.txt", "absent\n")?;
        let feat_before = ref_id(&repo, FEAT_REF)?;
        let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
        assert_commit_denied(
            commit_to_ref(&mut ctx, FEAT_REF, "absent config", DryRun::No),
            "config.invalid",
        );
        assert_eq!(ref_id(&repo, FEAT_REF)?, feat_before);

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
        if let Some(new_commit) = outcome.new_commit {
            assert!(
                repo.find_object(new_commit).is_err(),
                "allowed dry run commit object must not be persisted in the live repo"
            );
        }
        println!("malformed and absent config return `config.invalid`");
        println!("denied and allowed DryRun commits persist no ref/object");
        Ok(())
    })?;

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
