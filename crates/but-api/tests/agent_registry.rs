use std::{ffi::OsString, path::PathBuf};

use anyhow::Context as _;
use but_api::{
    commit::create::gate::{CommitGateTarget, enforce_commit_gate_for_target},
    legacy::{
        config_mutate::enforce_administration_write_gate, forge::approve_review,
        merge_gate::enforce_merge_gate,
    },
};
use but_authz::{Denial, Registry};
use but_db::ForgeReview;

const MAIN_REF: &str = "refs/heads/main";
const FEAT_REF: &str = "refs/heads/feat";
const REVIEW_ID: usize = 1;

#[test]
#[serial_test::serial]
fn commit_surface() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo("dev", "contents:write");
    let registry = RegistryEnv::registered("dev")?;
    let target = CommitGateTarget::config_only(gix::refs::FullName::try_from(MAIN_REF)?);

    enforce_commit_gate_for_target(&repo, &target)?;
    println!("commit_surface registered principal `dev` authorized with Ok(())");

    registry.unregister_current()?;
    let err = enforce_commit_gate_for_target(&repo, &target)
        .expect_err("unregistered commit surface must deny the current process");
    assert_perm_denied(&err, "commit_surface");
    println!("commit_surface unregistered process denied with `perm.denied`");

    Ok(())
}

#[test]
#[serial_test::serial]
fn merge_surface() -> anyhow::Result<()> {
    let (repo, _tmp) = merge_repo();
    let ctx = context_with_review(&repo, ref_id(&repo, FEAT_REF)?)?;
    let registry = RegistryEnv::registered("merger")?;

    enforce_merge_gate(&ctx, REVIEW_ID)?;
    println!("merge_surface registered principal `merger` authorized with Ok(())");

    registry.unregister_current()?;
    let err = enforce_merge_gate(&ctx, REVIEW_ID)
        .expect_err("unregistered merge surface must deny the current process");
    assert_perm_denied(&err, "merge_surface");
    println!("merge_surface unregistered process denied with `perm.denied`");

    Ok(())
}

#[test]
#[serial_test::serial]
fn admin_write_surface() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo("admin", "administration:write");
    let registry = RegistryEnv::registered("admin")?;

    enforce_administration_write_gate(&repo, MAIN_REF)?;
    println!("admin_write_surface registered principal `admin` authorized with Ok(())");

    registry.unregister_current()?;
    let err = enforce_administration_write_gate(&repo, MAIN_REF)
        .expect_err("unregistered admin-write surface must deny the current process");
    assert_perm_denied(&err, "admin_write_surface");
    println!("admin_write_surface unregistered process denied with `perm.denied`");

    Ok(())
}

#[test]
#[serial_test::serial]
fn forge_review_surface() -> anyhow::Result<()> {
    let (repo, _tmp) = forge_repo();
    let ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();
    let runtime = tokio::runtime::Runtime::new()?;
    let registry = RegistryEnv::registered("reviewer")?;

    runtime.block_on(approve_review(ctx.to_sync(), "feat".to_owned()))?;
    let verdicts = ctx
        .db
        .get_cache()?
        .local_review_verdicts()
        .list_by_target("feat")?;
    assert_eq!(
        verdicts.len(),
        1,
        "registered reviewer must write exactly one approval verdict"
    );
    assert_eq!(
        verdicts[0].principal_id, "reviewer",
        "approval verdict must be attributed to the registry-resolved principal"
    );
    println!("forge_review_surface registered principal `reviewer` authorized with Ok(Some)");

    registry.unregister_current()?;
    let err = runtime
        .block_on(approve_review(ctx.to_sync(), "feat".to_owned()))
        .expect_err("unregistered forge review surface must deny the current process");
    assert_perm_denied(&err, "forge_review_surface");
    println!("forge_review_surface unregistered process denied with `perm.denied`");

    Ok(())
}

struct RegistryEnv {
    path: PathBuf,
    _file: tempfile::NamedTempFile,
    previous_registry_path: Option<OsString>,
    previous_agent_handle: Option<OsString>,
    previous_allow_env_handle: Option<OsString>,
    pid: u32,
    start_time: u64,
}

impl RegistryEnv {
    fn registered(agent_id: &str) -> anyhow::Result<Self> {
        let file = tempfile::NamedTempFile::new()?;
        let path = file.path().to_owned();
        let pid = but_authz::current_pid();
        let start_time = but_authz::process_start_time(pid)?;
        let mut registry = Registry::empty();
        registry.register(pid, start_time, agent_id, 60, "operator")?;
        registry.write(&path)?;

        let env = Self {
            path,
            _file: file,
            previous_registry_path: std::env::var_os("BUT_AGENT_REGISTRY_PATH"),
            previous_agent_handle: std::env::var_os("BUT_AGENT_HANDLE"),
            previous_allow_env_handle: std::env::var_os("BUT_AUTHZ_ALLOW_ENV_HANDLE"),
            pid,
            start_time,
        };
        set_env_var("BUT_AGENT_REGISTRY_PATH", Some(env.path.as_os_str()));
        set_env_var("BUT_AGENT_HANDLE", None);
        set_env_var("BUT_AUTHZ_ALLOW_ENV_HANDLE", None);
        println!(
            "registered test process pid={} start_time={} as `{}` via BUT_AGENT_REGISTRY_PATH={}",
            env.pid,
            env.start_time,
            agent_id,
            env.path.display()
        );
        Ok(env)
    }

    fn unregister_current(&self) -> anyhow::Result<()> {
        let mut registry = Registry::load(&self.path)?;
        registry
            .unregister((self.pid, self.start_time))
            .with_context(|| {
                format!(
                    "registry should contain pid={} start_time={}",
                    self.pid, self.start_time
                )
            })?;
        registry.write(&self.path)?;
        println!(
            "unregistered test process pid={} start_time={} from {}",
            self.pid,
            self.start_time,
            self.path.display()
        );
        Ok(())
    }
}

impl Drop for RegistryEnv {
    fn drop(&mut self) {
        restore_env_var(
            "BUT_AGENT_REGISTRY_PATH",
            self.previous_registry_path.take(),
        );
        restore_env_var("BUT_AGENT_HANDLE", self.previous_agent_handle.take());
        restore_env_var(
            "BUT_AUTHZ_ALLOW_ENV_HANDLE",
            self.previous_allow_env_handle.take(),
        );
    }
}

fn governed_repo(principal: &str, authority: &str) -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        &format!(
            r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "{principal}"
permissions = ["{authority}"]
EOF

cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = false

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
"#
        ),
        &repo,
    );
    (repo, tmp)
}

fn merge_repo() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
git remote add origin https://github.com/gitbutler/agent-registry-surface-fixture.git
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "impl"
permissions = ["contents:write", "reviews:write"]

[[principal]]
id = "merger"
permissions = ["merge"]
EOF

cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
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

fn forge_repo() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "reviewer"
permissions = ["reviews:write"]
EOF

cat >.gitbutler/gates.toml <<'EOF'
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

fn context_with_review(
    repo: &gix::Repository,
    head: gix::ObjectId,
) -> anyhow::Result<but_ctx::Context> {
    let ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
    ctx.db
        .get_cache_mut()?
        .forge_reviews_mut()?
        .upsert(ForgeReview {
            html_url: "https://github.com/gitbutler/agent-registry-surface-fixture/pull/1"
                .to_owned(),
            number: REVIEW_ID.try_into()?,
            title: "Agent registry surface fixture".to_owned(),
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
                "https://github.com/gitbutler/agent-registry-surface-fixture.git".to_owned(),
            ),
            repo_owner: Some("gitbutler".to_owned()),
            head_repo_is_fork: false,
            reviewers: "[]".to_owned(),
            unit_symbol: "#".to_owned(),
            last_sync_at: fixed_time(),
            struct_version: but_forge::ForgeReview::struct_version(),
        })?;
    Ok(ctx)
}

fn fixed_time() -> chrono::NaiveDateTime {
    chrono::DateTime::from_timestamp(1_735_689_600, 0)
        .expect("fixed timestamp is valid")
        .naive_utc()
}

fn ref_id(repo: &gix::Repository, ref_name: &str) -> anyhow::Result<gix::ObjectId> {
    Ok(repo.find_reference(ref_name)?.peel_to_id()?.detach())
}

fn assert_perm_denied(err: &anyhow::Error, surface: &str) {
    let denial = err
        .downcast_ref::<Denial>()
        .unwrap_or_else(|| panic!("{surface} denial must downcast to but_authz::Denial"));
    assert_eq!(
        denial.code,
        Denial::PERM_DENIED_CODE,
        "{surface} denial must use the stable perm.denied code"
    );
}

fn set_env_var(key: &str, value: Option<&std::ffi::OsStr>) {
    unsafe {
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}

fn restore_env_var(key: &str, value: Option<OsString>) {
    unsafe {
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}
