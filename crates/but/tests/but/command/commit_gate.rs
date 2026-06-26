use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::Duration,
};

use anyhow::Context as _;

use crate::utils::{CommandExt as _, Sandbox};

// Governance is OPT-IN BY PRESENCE (RF-010 amended): a repo with NO committed
// `.gitbutler/*.toml` is ungoverned, so a commit lands with no handle and no
// authorization. This is INTENDED behavior, not a fail-open -- governance
// activates only once config is committed (proven by the activation transition in
// `commit_gate_opt_in_activation_denies_after_config` and the governed-deny tests
// `commit_gate_denies_protected_branch` / `commit_gate_denies_new_branch_without_contents_write`).
#[test]
fn commit_gate_allows_non_governed_commit2_flow() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack")?;
    env.setup_metadata(&["A"])?;

    env.file("ordinary.txt", "ordinary");
    env.but("commit2 -m ordinary").assert().success();

    env.but("status")
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
╭┄zz [unassigned changes] (no changes)
┊
┊╭┄g0 [A]
┊●   [..] ordinary
┊●   9477ae7 add A
├╯
┊
┴ 0dc3733 (common base) 2000-01-02 add M

Hint: run `but help` for all commands

"#]]);

    Ok(())
}

// Opt-in ACTIVATION: the SAME repo flips from allowed to denied once governance is
// committed at the target ref. First an ungoverned commit lands; then committing a
// governance config that protects branch `A` makes the next direct commit to `A`
// deny `branch.protected`. This is the transition neither the pure ungoverned nor
// the pure governed fixtures exercise.
#[test]
fn commit_gate_opt_in_activation_denies_after_config() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack")?;
    env.setup_metadata(&["A"])?;

    // 1. Ungoverned: a commit lands (no governance config committed yet).
    env.file("ordinary.txt", "ordinary");
    env.but("commit2 -m ordinary").assert().success();

    // 2. Commit a governance config onto branch `A` that protects `A` and grants
    //    `dev` contents:write (so the denial is branch.protected, not perm.denied).
    env.invoke_bash(
        r#"
base=$(git rev-parse refs/heads/A)
index=$(mktemp)
export GIT_INDEX_FILE="$index"
git read-tree "$base"
permissions_blob=$(git hash-object -w --stdin <<'EOF'
[[principal]]
id = "dev"
permissions = ["contents:write"]
EOF
)
gates_blob=$(git hash-object -w --stdin <<'EOF'
[[branch]]
name = "A"
protected = true
EOF
)
git update-index --add --cacheinfo 100644 "$permissions_blob" .gitbutler/permissions.toml
git update-index --add --cacheinfo 100644 "$gates_blob" .gitbutler/gates.toml
tree=$(git write-tree)
commit=$(printf 'activate governance\n' | git commit-tree "$tree" -p "$base")
git update-ref refs/heads/A "$commit"
rm "$index"
unset GIT_INDEX_FILE
"#,
    );

    // 3. Now governed: a direct commit to protected `A` is denied branch.protected.
    env.file("after-governance.txt", "after");
    env.but("--format json commit2 -m after-governance")
        .allow_json()
        .env("BUT_AUTHZ_ALLOW_ENV_HANDLE", "1")
        .env("BUT_AGENT_HANDLE", "dev")
        .assert()
        .failure()
        .stderr_eq(snapbox::str![[r#"
Error: {"error":{"code":"branch.protected","message":[..]A[..]}}

"#]]);

    Ok(())
}

#[test]
fn commit_gate_denies_protected_branch() -> anyhow::Result<()> {
    let env = governed_env("one-stack", Some("A"))?;

    env.file("protected.txt", "protected");
    env.but("--format json commit2 -m protected")
        .allow_json()
        .env("BUT_AUTHZ_ALLOW_ENV_HANDLE", "1").env("BUT_AGENT_HANDLE", "dev")
        .assert()
        .failure()
        .stdout_eq(snapbox::str![[r#"
"#]])
        .stderr_eq(snapbox::str![[r#"
Error: {"error":{"code":"branch.protected","message":"direct commits to protected branch [..]A[..] are denied for principal [..]dev[..]; land changes through a reviewed merge"}}

"#]]);

    Ok(())
}

#[test]
fn commit_gate_denies_new_branch_without_contents_write() -> anyhow::Result<()> {
    let env = governed_env("zero-stacks", None)?;

    env.file("readonly.txt", "readonly");
    env.but("--format json commit2 -m readonly -b")
        .allow_json()
        .env("BUT_AUTHZ_ALLOW_ENV_HANDLE", "1").env("BUT_AGENT_HANDLE", "ro")
        .assert()
        .failure()
        .stdout_eq(snapbox::str![[r#"
"#]])
        .stderr_eq(snapbox::str![[r#"
Error: {"error":{"code":"perm.denied","message":"action requires contents:write; authorization denied (held permissions: contents:read)"}}

"#]]);

    Ok(())
}

#[test]
fn commit_gate_reports_invalid_config_for_commit_relative_target() -> anyhow::Result<()> {
    let env = Sandbox::open_scenario_with_target_and_default_settings("one-stack-two-commits")?;
    env.invoke_bash(
        r#"
base=$(git rev-parse refs/heads/A)
index=$(mktemp)
export GIT_INDEX_FILE="$index"
git read-tree "$base"
permissions_blob=$(git hash-object -w --stdin <<'EOF'
[[principal]]
id = "dev"
permissions = ["contents:write"]
EOF
)
gates_blob=$(git hash-object -w --stdin <<'EOF'
[[branch]
name = "A"
protected = nope
EOF
)
git update-index --add --cacheinfo 100644 "$permissions_blob" .gitbutler/permissions.toml
git update-index --add --cacheinfo 100644 "$gates_blob" .gitbutler/gates.toml
tree=$(git write-tree)
commit=$(printf 'malformed governance\n' | git commit-tree "$tree" -p "$base")
git update-ref refs/heads/A "$commit"
rm "$index"
unset GIT_INDEX_FILE
"#,
    );
    env.but("setup").assert().success();
    env.setup_metadata(&["A"])?;

    env.file("invalid.txt", "invalid");
    env.but("--format json commit2 -m invalid --above fe")
        .allow_json()
        .env("BUT_AUTHZ_ALLOW_ENV_HANDLE", "1").env("BUT_AGENT_HANDLE", "dev")
        .assert()
        .failure()
        .stdout_eq(snapbox::str![[r#"
"#]])
        .stderr_eq(snapbox::str![[r#"
Error: {"error":{"code":"config.invalid","message":"invalid governance config: parsing .gitbutler/gates.toml at refs/heads/A[..]"}}

"#]]);

    Ok(())
}

#[test]
#[serial_test::serial]
fn commit_gate_operator_runtime_registry_sequence() -> anyhow::Result<()> {
    let env = governed_operator_env()?;
    let registry_path = env.projects_root().join("operator-runtime-registry.toml");

    env.file("env-only-denied.txt", "env-only");
    env.but("--format json commit feat -m env-only-denied")
        .allow_json()
        .env("BUT_AGENT_REGISTRY_PATH", &registry_path)
        .env("BUT_AUTHZ_ALLOW_ENV_HANDLE", "1")
        .env("BUT_AGENT_HANDLE", "dev")
        .env_remove("BUT_AUTHZ_ALLOW_ENV_HANDLE")
        .assert()
        .failure()
        .stderr_eq(snapbox::str![[r#"
[..]perm.denied[..]
"#]]);
    println!("first env-only commit stderr contains literal `perm.denied`");

    env.but("--format json commit feat -m flag-fallback")
        .allow_json()
        .env("BUT_AGENT_REGISTRY_PATH", &registry_path)
        .env("BUT_AUTHZ_ALLOW_ENV_HANDLE", "1")
        .env("BUT_AGENT_HANDLE", "dev")
        .env("BUT_AUTHZ_ALLOW_ENV_HANDLE", "1")
        .assert()
        .success();
    println!("flag-set env fallback commit exits 0");

    env.but("agent register --as dev --ttl 4h")
        .env("BUT_AGENT_REGISTRY_PATH", &registry_path)
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
registered: pid=[..] start_time=[..] agent_id=dev expires_at=[..]

"#]]);
    println!("agent register stdout contains registered dev pid/start_time tuple");

    env.file("registered-no-env.txt", "registered");
    let stopped_commit = spawn_stopped_but_commit(&env, &registry_path, "registered-no-env")?;
    env.but(format!(
        "agent register --pid {} --start-time {} --as dev --ttl 4h",
        stopped_commit.pid, stopped_commit.start_time
    ))
    .env("BUT_AGENT_REGISTRY_PATH", &registry_path)
    .assert()
    .success()
    .stdout_eq(snapbox::str![[r#"
registered: pid=[..] start_time=[..] agent_id=dev expires_at=[..]

"#]]);
    continue_process(stopped_commit.pid)?;
    let registered_output = stopped_commit.child.wait_with_output()?;
    assert!(
        registered_output.status.success(),
        "registered no-env commit must exit 0; stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&registered_output.stdout),
        String::from_utf8_lossy(&registered_output.stderr)
    );
    println!("registered no-env commit exits 0");

    env.but(format!(
        "agent unregister --pid {} --start-time {}",
        stopped_commit.pid, stopped_commit.start_time
    ))
    .env("BUT_AGENT_REGISTRY_PATH", &registry_path)
    .assert()
    .success()
    .stdout_eq(snapbox::str![[r#"
removed: pid=[..] start_time=[..]

"#]]);
    println!("but agent unregister succeeds");

    env.file("post-unregister-denied.txt", "post-unregister");
    env.but("--format json commit feat -m post-unregister-denied")
        .allow_json()
        .env("BUT_AGENT_REGISTRY_PATH", &registry_path)
        .env("BUT_AUTHZ_ALLOW_ENV_HANDLE", "1")
        .env("BUT_AGENT_HANDLE", "dev")
        .env_remove("BUT_AUTHZ_ALLOW_ENV_HANDLE")
        .assert()
        .failure()
        .stderr_eq(snapbox::str![[r#"
[..]perm.denied[..]
"#]]);
    println!("post-unregister env-only commit stderr contains literal `perm.denied`");

    Ok(())
}

// C1 keystone (XDG mode): default-path register -> gate round-trip with NONE of
// `BUT_AGENT_REGISTRY_PATH` / `BUT_AGENT_HANDLE` / `BUT_AUTHZ_ALLOW_ENV_HANDLE`
// set. This exercises the REAL runtime registry path resolution shared by the
// CLI writer (`but agent register`) and the gate reader. The CLI and the gate
// must resolve the SAME default file: `$XDG_RUNTIME_DIR/gitbutler/<repo-hash>/
// agents-runtime.toml`. Before the resolver was unified, the CLI writer and the
// gate reader used two different filenames, so a registered process was
// invisible to the gate in the field (registry-miss -> perm.denied). Every
// other gate test pins `BUT_AGENT_REGISTRY_PATH` to one shared file, masking
// the divergence; here the default path is resolved for real, so the registered
// commit must exit 0.
#[test]
#[serial_test::serial]
fn commit_gate_default_xdg_registry_path_register_then_commit_allowed() -> anyhow::Result<()> {
    let env = governed_operator_env()?;
    let xdg_runtime_dir = tempfile::tempdir().context("creating XDG_RUNTIME_DIR tempdir")?;

    env.file("registered-default-xdg.txt", "registered-default-xdg");
    let stopped_commit = spawn_stopped_but_commit_default_registry(
        &env,
        "registered-default-xdg",
        Some(xdg_runtime_dir.path()),
    )?;
    env.but(format!(
        "agent register --pid {} --start-time {} --as dev --ttl 4h",
        stopped_commit.pid, stopped_commit.start_time
    ))
    .env("XDG_RUNTIME_DIR", xdg_runtime_dir.path())
    .assert()
    .success()
    .stdout_eq(snapbox::str![[r#"
registered: pid=[..] start_time=[..] agent_id=dev expires_at=[..]

"#]]);

    let written = find_files_named(xdg_runtime_dir.path(), "agents-runtime.toml");
    assert_eq!(
        written.len(),
        1,
        "but agent register must write exactly one agents-runtime.toml under XDG_RUNTIME_DIR for the gate to read; found {written:?}"
    );

    continue_process(stopped_commit.pid)?;
    let registered_output = stopped_commit.child.wait_with_output()?;
    assert!(
        registered_output.status.success(),
        "default-path (XDG) register->commit must exit 0; stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&registered_output.stdout),
        String::from_utf8_lossy(&registered_output.stderr)
    );
    println!("default XDG-path register->commit exits 0 with no env overrides");
    drop(xdg_runtime_dir);

    Ok(())
}

// C1 keystone (macOS workdir-fallback mode): same default-path round-trip with
// NO env overrides AND no `XDG_RUNTIME_DIR`, so resolution falls back to
// `<workdir>/.gitbutler/agents-runtime.toml` (the macOS field path). The CLI
// writer and the gate reader must resolve the same workdir file; the registered
// commit must exit 0.
#[test]
#[serial_test::serial]
fn commit_gate_default_workdir_registry_path_register_then_commit_allowed() -> anyhow::Result<()> {
    let env = governed_operator_env()?;

    env.file(
        "registered-default-workdir.txt",
        "registered-default-workdir",
    );
    let stopped_commit =
        spawn_stopped_but_commit_default_registry(&env, "registered-default-workdir", None)?;
    env.but(format!(
        "agent register --pid {} --start-time {} --as dev --ttl 4h",
        stopped_commit.pid, stopped_commit.start_time
    ))
    .env_remove("XDG_RUNTIME_DIR")
    .assert()
    .success()
    .stdout_eq(snapbox::str![[r#"
registered: pid=[..] start_time=[..] agent_id=dev expires_at=[..]

"#]]);

    let registry_file = env
        .projects_root()
        .join(".gitbutler")
        .join("agents-runtime.toml");
    assert!(
        registry_file.exists(),
        "but agent register (no XDG) must write {} for the gate to read",
        registry_file.display()
    );

    continue_process(stopped_commit.pid)?;
    let registered_output = stopped_commit.child.wait_with_output()?;
    assert!(
        registered_output.status.success(),
        "default-path (workdir fallback) register->commit must exit 0; stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&registered_output.stdout),
        String::from_utf8_lossy(&registered_output.stderr)
    );
    println!("default workdir-fallback register->commit exits 0 with no env overrides");

    Ok(())
}

fn governed_env(name: &str, stack: Option<&str>) -> anyhow::Result<Sandbox> {
    let env = Sandbox::open_scenario_with_target_and_default_settings(name)?;
    env.invoke_bash(
        r#"
write_governance_commit() {
    target_ref="$1"
    base=$(git rev-parse "$target_ref")
    index=$(mktemp)
    export GIT_INDEX_FILE="$index"
    git read-tree "$base"
    permissions_blob=$(git hash-object -w --stdin <<'EOF'
[[principal]]
id = "dev"
permissions = ["contents:write"]

[[principal]]
id = "ro"
permissions = ["contents:read"]
EOF
)
    gates_blob=$(git hash-object -w --stdin <<'EOF'
[[branch]]
name = "main"
protected = true

[[branch]]
name = "A"
protected = true
EOF
)
    git update-index --add --cacheinfo 100644 "$permissions_blob" .gitbutler/permissions.toml
    git update-index --add --cacheinfo 100644 "$gates_blob" .gitbutler/gates.toml
    tree=$(git write-tree)
    commit=$(printf 'governance config\n' | git commit-tree "$tree" -p "$base")
    git update-ref "$target_ref" "$commit"
    rm "$index"
    unset GIT_INDEX_FILE
}

write_governance_commit refs/heads/main
if git show-ref --verify --quiet refs/remotes/origin/main
then
    write_governance_commit refs/remotes/origin/main
fi
if git show-ref --verify --quiet refs/heads/A
then
    write_governance_commit refs/heads/A
fi
"#,
    );
    env.but("setup").assert().success();
    match stack {
        Some(branch_name) => {
            env.setup_metadata(&[branch_name])?;
        }
        None => {
            env.setup_metadata(&[])?;
        }
    }
    Ok(env)
}

fn governed_operator_env() -> anyhow::Result<Sandbox> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack")?;
    env.invoke_bash(
        r#"
git branch -f main origin/main
git branch -m A feat
write_governance_commit() {
    target_ref="$1"
    base=$(git rev-parse "$target_ref")
    index=$(mktemp)
    export GIT_INDEX_FILE="$index"
    git read-tree "$base"
    permissions_blob=$(git hash-object -w --stdin <<'EOF'
[[principal]]
id = "dev"
permissions = ["contents:write"]
EOF
)
    agents_blob=$(git hash-object -w --stdin <<'EOF'
[[agent]]
id = "dev"
permissions = ["contents:write"]
EOF
)
    gates_blob=$(git hash-object -w --stdin <<'EOF'
[[branch]]
name = "feat"
protected = false
EOF
)
    git update-index --add --cacheinfo 100644 "$permissions_blob" .gitbutler/permissions.toml
    git update-index --add --cacheinfo 100644 "$agents_blob" .gitbutler/agents.toml
    git update-index --add --cacheinfo 100644 "$gates_blob" .gitbutler/gates.toml
    tree=$(git write-tree)
    commit=$(printf 'operator governance config\n' | git commit-tree "$tree" -p "$base")
    git update-ref "$target_ref" "$commit"
    rm "$index"
    unset GIT_INDEX_FILE
}

write_governance_commit refs/heads/main
write_governance_commit refs/heads/feat
git checkout feat
"#,
    );
    env.but("setup").assert().success();
    env.set_target_sha("refs/heads/main")?;
    env.setup_metadata(&["feat"])?;
    env.but("apply feat").assert().success();
    Ok(env)
}

struct StoppedButCommit {
    child: Child,
    pid: u32,
    start_time: u64,
}

fn spawn_stopped_but_commit(
    env: &Sandbox,
    registry_path: &Path,
    message: &str,
) -> anyhow::Result<StoppedButCommit> {
    let but_bin = snapbox::cmd::cargo_bin!("but");
    let child = Command::new("sh")
        .arg("-c")
        .arg(
            r#"
kill -STOP $$
exec "$BUT_BIN" --format json commit feat -m "$COMMIT_MESSAGE"
"#,
        )
        .env("BUT_BIN", but_bin)
        .env("COMMIT_MESSAGE", message)
        .env("BUT_AGENT_REGISTRY_PATH", registry_path)
        .env("BUT_AUTHZ_ALLOW_ENV_HANDLE", "1")
        .env_remove("BUT_AGENT_HANDLE")
        .env_remove("BUT_AUTHZ_ALLOW_ENV_HANDLE")
        .env_remove("BUT_OUTPUT_FORMAT")
        .env("E2E_TEST_APP_DATA_DIR", env.app_data_dir())
        .env("GITBUTLER_CHANGE_ID", "42")
        .env("NOPAGER", "1")
        .current_dir(env.projects_root())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("spawning stopped but commit process")?;
    let pid = child.id();
    let start_time = process_start_time_retry(pid)?;
    Ok(StoppedButCommit {
        child,
        pid,
        start_time,
    })
}

// Spawn a stopped `but commit` child that resolves the DEFAULT runtime registry
// path (no `BUT_AGENT_REGISTRY_PATH`). `xdg_runtime_dir` selects the resolution
// mode: `Some` sets `XDG_RUNTIME_DIR` (XDG mode), `None` removes it (workdir
// fallback). The same selection must be applied to the paired `but agent
// register` call so both halves resolve the same file.
fn spawn_stopped_but_commit_default_registry(
    env: &Sandbox,
    message: &str,
    xdg_runtime_dir: Option<&Path>,
) -> anyhow::Result<StoppedButCommit> {
    let but_bin = snapbox::cmd::cargo_bin!("but");
    let mut command = Command::new("sh");
    command
        .arg("-c")
        .arg(
            r#"
kill -STOP $$
exec "$BUT_BIN" --format json commit feat -m "$COMMIT_MESSAGE"
"#,
        )
        .env("BUT_BIN", but_bin)
        .env("COMMIT_MESSAGE", message)
        .env_remove("BUT_AGENT_REGISTRY_PATH")
        .env_remove("BUT_AGENT_HANDLE")
        .env_remove("BUT_AUTHZ_ALLOW_ENV_HANDLE")
        .env_remove("BUT_OUTPUT_FORMAT")
        .env("E2E_TEST_APP_DATA_DIR", env.app_data_dir())
        .env("GITBUTLER_CHANGE_ID", "42")
        .env("NOPAGER", "1")
        .current_dir(env.projects_root())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    match xdg_runtime_dir {
        Some(dir) => command.env("XDG_RUNTIME_DIR", dir),
        None => command.env_remove("XDG_RUNTIME_DIR"),
    };
    let child = command
        .spawn()
        .context("spawning stopped but commit process with default registry path")?;
    let pid = child.id();
    let start_time = process_start_time_retry(pid)?;
    Ok(StoppedButCommit {
        child,
        pid,
        start_time,
    })
}

fn find_files_named(root: &Path, name: &str) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(root) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(find_files_named(&path, name));
        } else if path.file_name().and_then(|name| name.to_str()) == Some(name) {
            found.push(path);
        }
    }
    found
}

fn process_start_time_retry(pid: u32) -> anyhow::Result<u64> {
    let mut last_error = None;
    for _ in 0..50 {
        match but_authz::process_start_time(pid) {
            Ok(start_time) => return Ok(start_time),
            Err(error) => {
                last_error = Some(error);
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
    Err(last_error
        .unwrap_or_else(|| anyhow::anyhow!("process_start_time({pid}) was not attempted")))
}

fn continue_process(pid: u32) -> anyhow::Result<()> {
    let status = Command::new("kill")
        .arg("-CONT")
        .arg(pid.to_string())
        .status()
        .with_context(|| format!("continuing stopped but process pid {pid}"))?;
    anyhow::ensure!(
        status.success(),
        "kill -CONT {pid} must succeed before waiting for registered commit"
    );
    Ok(())
}
