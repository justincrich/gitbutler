//! STEER-004 — class mapping + branch-protected wiring + operator-required
//! steering payload integration proofs.
//!
//! This suite proves the STEER-004 wiring against a real git repository:
//!
//! - TC-1: class is correct per (code, principal-resolution) — a resolved
//!   principal's `perm.denied` is `actor_correctable`.
//! - TC-3: `config.invalid` carries `authorized_actions == []` + a
//!   do-not-retry `do_not` (operator_required).
//! - TC-4: `branch.protected` threads `&cfg`, so `held_permissions` is
//!   re-derived (includes `contents:write`) and the menu offers a
//!   feature-branch commit.
//! - TC-6/7: a no-lateral actor_correctable denial degrades to the
//!   vertical path (empty/discovery-only menu, remediation_hint naming a
//!   grant/handoff); `do_not` does not enumerate bypass mechanics.
//! - TC-8: a DryRun `branch.protected` denial carries the full steering
//!   payload while persisting nothing (no new git objects, no ref advance).
//!
//! See `.spec/prds/governance/tasks/sprint-08-steer-capability-aware-denials/
//! STEER-004-class-mapping-branch-protected-wiring.md` for the task contract.

use but_core::{DiffSpec, DryRun};
use but_rebase::graph_rebase::mutate::{InsertSide, RelativeTo};

const MAIN_REF: &str = "refs/heads/main";
const FEAT_REF: &str = "refs/heads/feat";

// ---------------------------------------------------------------------------
// TC-1 — class is correct per (code, principal-resolution) [PRIMARY AC-1]
// ---------------------------------------------------------------------------

/// AC-1 / TC-1 — a resolved-principal missing-authority `perm.denied`
/// carries `class:"actor_correctable"` + populated `held_permissions`.
///
/// GIVEN the `ro` principal (holds only `contents:read`) is denied a commit
/// to feat, WHEN the commit gate returns `perm.denied`, THEN the denial
/// carries `class:"actor_correctable"` and `held_permissions` includes
/// `contents:read`.
#[test]
#[serial_test::serial]
fn steer_class_per_code_and_resolution() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let feat_before = ref_id(&repo, FEAT_REF)?;

    temp_env::with_vars(
        [
            ("BUT_AGENT_HANDLE", Some("ro")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || -> anyhow::Result<()> {
            checkout(&repo, "feat");
            write_file(&repo, "ro-change.txt", "ro\n")?;
            let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
            let gate_error = assert_commit_denied(
                commit_to_ref(&mut ctx, FEAT_REF, "ro commit", DryRun::No),
                "perm.denied",
            );

            // AC-1: resolved principal → actor_correctable.
            assert_eq!(
                gate_error.class,
                but_authz::DenialClass::ActorCorrectable,
                "resolved-principal perm.denied MUST be actor_correctable"
            );

            // held_permissions must include contents:read (the ro principal's
            // sole authority) — not empty.
            assert!(
                gate_error
                    .held_permissions
                    .contains(&but_authz::Authority::ContentsRead),
                "perm.denied held_permissions MUST include contents:read for the ro principal: {:?}",
                gate_error.held_permissions
            );

            // Must NOT observe: operator_required class.
            assert_ne!(
                gate_error.class,
                but_authz::DenialClass::OperatorRequired,
                "resolved-principal perm.denied MUST NOT be operator_required"
            );

            assert_eq!(
                ref_id(&repo, FEAT_REF)?,
                feat_before,
                "perm.denied must leave feat unchanged"
            );

            println!(
                "AC-1: ro perm.denied → class:\"actor_correctable\", held_permissions includes contents:read"
            );
            Ok(())
        },
    )?;

    Ok(())
}

// ---------------------------------------------------------------------------
// TC-3 — config.invalid → empty menu + do-not-retry do_not [AC-2]
// ---------------------------------------------------------------------------

/// AC-2 / TC-3 — a `config.invalid` denial carries `authorized_actions == []`
/// and a `do_not` that says do-not-retry / requires an operator.
///
/// GIVEN a malformed committed `gates.toml` at the target ref, WHEN a gated
/// action runs, THEN the denial is `config.invalid` with
/// `authorized_actions == []` and a `do_not` containing "do not retry".
#[test]
#[serial_test::serial]
fn steer_operator_required_empty_menu_do_not() -> anyhow::Result<()> {
    temp_env::with_vars(
        [
            ("BUT_AGENT_HANDLE", Some("dev")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || -> anyhow::Result<()> {
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
            let gate_error = assert_commit_denied(
                commit_to_ref(&mut ctx, FEAT_REF, "malformed config", DryRun::No),
                "config.invalid",
            );

            // AC-2: config.invalid → operator_required.
            assert_eq!(
                gate_error.class,
                but_authz::DenialClass::OperatorRequired,
                "config.invalid MUST be operator_required"
            );

            // Must observe: empty authorized_actions.
            assert!(
                gate_error.authorized_actions.is_empty(),
                "config.invalid authorized_actions MUST be empty ([]): {:?}",
                gate_error.authorized_actions
            );

            // Must observe: do_not containing "do not retry".
            let do_not = gate_error
                .do_not
                .expect("config.invalid MUST carry a do_not");
            assert!(
                do_not.contains("do not retry"),
                "config.invalid do_not MUST contain 'do not retry': {do_not}"
            );

            // Must NOT observe: actor_correctable class.
            assert_ne!(
                gate_error.class,
                but_authz::DenialClass::ActorCorrectable,
                "config.invalid MUST NOT be actor_correctable"
            );

            // Must NOT observe: populated held_permissions on config.invalid path.
            assert!(
                gate_error.held_permissions.is_empty(),
                "config.invalid held_permissions MUST be empty: {:?}",
                gate_error.held_permissions
            );

            assert_eq!(ref_id(&repo, FEAT_REF)?, feat_before);
            println!(
                "AC-2: config.invalid → operator_required, authorized_actions==[], do_not says do-not-retry"
            );
            Ok(())
        },
    )?;

    Ok(())
}

// ---------------------------------------------------------------------------
// TC-4 — branch.protected threads &cfg [AC-3]
// ---------------------------------------------------------------------------

/// AC-3 / TC-4 — a `branch.protected` denial carries `held_permissions`
/// including `contents:write` (re-derived via the threaded `&cfg`).
///
/// GIVEN the `dev` principal (holds `contents:write`) is denied a direct
/// commit to protected main, WHEN the commit gate returns
/// `branch.protected`, THEN the denial carries `held_permissions` including
/// `contents:write` (re-derived, not dropped) and an `authorized_actions`
/// entry whose command is `but commit` (a feature-branch affordance, not
/// the protected-ref commit).
#[test]
#[serial_test::serial]
fn steer_branch_protected_threads_cfg() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let main_before = ref_id(&repo, MAIN_REF)?;

    temp_env::with_vars(
        [
            ("BUT_AGENT_HANDLE", Some("dev")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || -> anyhow::Result<()> {
            checkout(&repo, "main");
            write_file(&repo, "main.txt", "main\n")?;
            let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
            let gate_error = assert_commit_denied(
                commit_to_ref(&mut ctx, MAIN_REF, "direct main commit", DryRun::No),
                "branch.protected",
            );

            // AC-3: class is actor_correctable.
            assert_eq!(
                gate_error.class,
                but_authz::DenialClass::ActorCorrectable,
                "branch.protected MUST be actor_correctable"
            );

            // AC-3: held_permissions includes contents:write (re-derived via &cfg).
            assert!(
                gate_error
                    .held_permissions
                    .contains(&but_authz::Authority::ContentsWrite),
                "branch.protected held_permissions MUST include contents:write (re-derived): {:?}",
                gate_error.held_permissions
            );

            // AC-3: authorized_actions offers a feature-branch commit, not the
            // protected-ref commit.
            let commands: Vec<&str> = gate_error
                .authorized_actions
                .iter()
                .map(|a| a.command)
                .collect();
            assert!(
                commands.contains(&"but commit"),
                "branch.protected menu MUST include `but commit` (feature-branch affordance): {commands:?}"
            );

            // The commit affordance effect must name an unprotected branch.
            let commit_action = gate_error
                .authorized_actions
                .iter()
                .find(|a| a.command == "but commit")
                .expect("`but commit` must be in the menu");
            assert!(
                commit_action.effect.to_lowercase().contains("unprotected"),
                "`but commit` effect MUST name an UNPROTECTED feature branch: {:?}",
                commit_action.effect
            );

            // The deny decision is unchanged.
            assert_eq!(
                ref_id(&repo, MAIN_REF)?,
                main_before,
                "branch.protected must leave main unchanged"
            );

            println!(
                "AC-3: branch.protected → held_permissions includes contents:write, menu offers feature-branch commit"
            );
            Ok(())
        },
    )?;

    Ok(())
}

// ---------------------------------------------------------------------------
// TC-6/7 — degrade to vertical path + do_not positive-only [AC-4]
// ---------------------------------------------------------------------------

/// AC-4 / TC-6, TC-7 — a no-lateral actor_correctable denial degrades to
/// the vertical path (discovery-only menu, remediation_hint naming a
/// grant/handoff), and any `do_not` is positive-only (does not enumerate
/// `git push` / `--no-verify` bypass mechanics).
///
/// GIVEN a resolved `ro` principal (holds only `contents:read`) is denied a
/// commit to feat, WHEN the denial is produced, THEN the menu is
/// discovery-only (`but perm list`, no fabricated lateral action) and the
/// remediation_hint names a grant/handoff path. Any `do_not` (when present)
/// must NOT mention `git push` or `--no-verify`.
#[test]
#[serial_test::serial]
fn steer_degrade_vertical_and_do_not_positive() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let feat_before = ref_id(&repo, FEAT_REF)?;

    temp_env::with_vars(
        [
            ("BUT_AGENT_HANDLE", Some("ro")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || -> anyhow::Result<()> {
            checkout(&repo, "feat");
            write_file(&repo, "ro-degrade.txt", "ro\n")?;
            let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();

            let result = commit_to_ref(&mut ctx, FEAT_REF, "ro degrade", DryRun::No);
            let err = match result {
                Ok(_) => panic!("commit should be denied with perm.denied"),
                Err(err) => err,
            };
            // Access the underlying Denial for remediation_hint (CommitGateError
            // does not expose it).
            let denial = err
                .downcast_ref::<but_authz::Denial>()
                .expect("perm.denied should be a structured Denial");
            assert_eq!(denial.code, "perm.denied");

            let gate_error = but_api::commit::create::gate::classify_error(&err)
                .expect("commit gate errors should be structured");

            // TC-6: remediation_hint names a grant/handoff path.
            assert!(
                denial.message.contains("contents:write"),
                "perm.denied message should name the missing authority"
            );
            assert!(
                denial.remediation_hint.contains("request a reviewed merge")
                    || denial.remediation_hint.contains("ask a maintainer"),
                "remediation_hint MUST name a grant/handoff path: {}",
                denial.remediation_hint
            );

            // TC-6: authorized_actions is empty or discovery-only (no fabricated
            // lateral action the ro principal cannot run).
            let commands: Vec<&str> = gate_error
                .authorized_actions
                .iter()
                .map(|a| a.command)
                .collect();
            for command in &commands {
                assert!(
                    *command == "but perm list",
                    "ro denial menu must be discovery-only — `{command}` is a lateral action ro cannot run: {commands:?}"
                );
            }

            // TC-7: do_not (when present) must NOT enumerate bypass mechanics.
            if let Some(do_not) = gate_error.do_not {
                let lower = do_not.to_lowercase();
                assert!(
                    !lower.contains("git push"),
                    "actor_correctable do_not MUST NOT mention 'git push' bypass: {do_not}"
                );
                assert!(
                    !lower.contains("--no-verify"),
                    "actor_correctable do_not MUST NOT mention '--no-verify' bypass: {do_not}"
                );
            }

            assert_eq!(
                ref_id(&repo, FEAT_REF)?,
                feat_before,
                "denial must leave feat unchanged"
            );

            println!(
                "AC-4: ro no-lateral → discovery-only menu, remediation_hint names grant path, do_not positive-only"
            );
            Ok(())
        },
    )?;

    Ok(())
}

// ---------------------------------------------------------------------------
// TC-8 — DryRun carries the full steering payload + no mutation [AC-5]
// ---------------------------------------------------------------------------

/// AC-5 / TC-8 — a DryRun `branch.protected` denial carries
/// `class`/`held_permissions`/`authorized_actions` AND mutates no
/// object/ref.
///
/// GIVEN a denied branch-protected commit run under DryRun, WHEN the denial
/// is produced, THEN the denial carries the full steering payload
/// (`class`, `held_permissions`, `authorized_actions`) AND no git object is
/// created, no ref is advanced.
#[test]
#[serial_test::serial]
fn steer_dryrun_full_payload_no_mutation() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let main_before = ref_id(&repo, MAIN_REF)?;
    let feat_before = ref_id(&repo, FEAT_REF)?;
    let object_count_before = object_count(&repo);

    temp_env::with_vars(
        [
            ("BUT_AGENT_HANDLE", Some("dev")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || -> anyhow::Result<()> {
            checkout(&repo, "main");
            write_file(&repo, "denied-dryrun.txt", "denied dry run\n")?;
            let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
            let gate_error = assert_commit_denied(
                commit_to_ref(&mut ctx, MAIN_REF, "denied dry run", DryRun::Yes),
                "branch.protected",
            );

            // AC-5: DryRun denial carries the full steering payload.
            assert_eq!(
                gate_error.class,
                but_authz::DenialClass::ActorCorrectable,
                "DryRun branch.protected MUST carry class"
            );
            assert!(
                !gate_error.held_permissions.is_empty(),
                "DryRun branch.protected MUST carry held_permissions: {:?}",
                gate_error.held_permissions
            );
            assert!(
                !gate_error.authorized_actions.is_empty(),
                "DryRun branch.protected MUST carry authorized_actions: {:?}",
                gate_error.authorized_actions
            );

            // AC-5: DryRun mutates no ref/object.
            assert_eq!(
                ref_id(&repo, MAIN_REF)?,
                main_before,
                "DryRun denial MUST leave main unchanged"
            );
            assert_eq!(
                ref_id(&repo, FEAT_REF)?,
                feat_before,
                "DryRun denial MUST leave feat unchanged"
            );
            assert_eq!(
                object_count(&repo),
                object_count_before,
                "DryRun denial MUST create no new git objects"
            );

            println!(
                "AC-5: DryRun branch.protected carries class/held_permissions/authorized_actions + no mutation"
            );
            Ok(())
        },
    )?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Fixtures + helpers (mirror crates/but-api/tests/commit_gate.rs)
// ---------------------------------------------------------------------------

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

fn ref_id(repo: &gix::Repository, ref_name: &str) -> anyhow::Result<gix::ObjectId> {
    Ok(repo.find_reference(ref_name)?.peel_to_id()?.detach())
}

fn object_count(repo: &gix::Repository) -> usize {
    let workdir = repo.workdir().expect("test repository must be non-bare");
    let output = std::process::Command::new("git")
        .args(["rev-list", "--objects", "--all"])
        .current_dir(workdir)
        .output()
        .expect("git rev-list should succeed");
    String::from_utf8_lossy(&output.stdout).lines().count()
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
