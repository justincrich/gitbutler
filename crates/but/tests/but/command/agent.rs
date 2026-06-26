use std::{fs, path::PathBuf};

use crate::utils::Sandbox;

macro_rules! snapshot {
    ($path:literal) => {
        include_str!($path)
    };
}

const RUST_IMPLEMENTER_AGENT: &str = r#"[[agent]]
id = "rust-implementer"
"#;

const RUST_IMPLEMENTER_AND_REVIEWER_AGENTS: &str = r#"[[agent]]
id = "rust-implementer"

[[agent]]
id = "rust-reviewer"
"#;

const LEGACY_PERMISSIONS_TOML: &str = r#"# preserve leading comments
[[principal]]
id = "rust-implementer"
permissions = ["contents:write"]

# role-based entries keep their body bytes
[[principal]]
id = "release-bot"
role = "maintain"
"#;

const MIGRATED_AGENTS_TOML: &str = r#"# preserve leading comments
[[agent]]
id = "rust-implementer"
permissions = ["contents:write"]

# role-based entries keep their body bytes
[[agent]]
id = "release-bot"
role = "maintain"
"#;

const AGENT_MIGRATE_REF_PIN_CAVEAT: &str =
    "Commit the add of .gitbutler/agents.toml and the delete of .gitbutler/permissions.toml together.";

#[test]
fn agent_help_lists_verbs() -> anyhow::Result<()> {
    let env = Sandbox::empty()?;

    env.but("agent --help")
        .assert()
        .success()
        .stdout_eq(snapshot!("snapshots/agent/help.stdout"));

    Ok(())
}

#[test]
fn agent_register_known_id_prints_tuple() -> anyhow::Result<()> {
    let env = ident_fixture("ident-agent-register-known", RUST_IMPLEMENTER_AGENT)?;
    let registry_path = registry_path(&env, "register-known");

    env.but("agent register --pid 12345 --start-time 1730000000 --as rust-implementer --ttl 1h")
        .env("BUT_AGENT_REGISTRY_PATH", &registry_path)
        .assert()
        .success()
        .stdout_eq(snapshot!("snapshots/agent/register-known.stdout"));

    Ok(())
}

#[test]
fn agent_register_unknown_id_rejects_without_registry_write() -> anyhow::Result<()> {
    let env = ident_fixture("ident-agent-register-unknown", RUST_IMPLEMENTER_AGENT)?;
    let registry_path = registry_path(&env, "register-unknown");

    env.but("agent register --pid 12345 --start-time 1730000000 --as ghost --ttl 1h")
        .env("BUT_AGENT_REGISTRY_PATH", &registry_path)
        .assert()
        .failure()
        .stdout_eq(snapbox::str![])
        .stderr_eq(snapshot!("snapshots/agent/register-unknown.stderr"));

    assert!(
        !registry_path.exists(),
        "unknown agent_id must fail before writing a runtime registry at {}",
        registry_path.display()
    );

    Ok(())
}

#[test]
fn agent_list_empty_and_populated_registries() -> anyhow::Result<()> {
    let env = ident_fixture("ident-agent-list", RUST_IMPLEMENTER_AND_REVIEWER_AGENTS)?;
    let registry_path = registry_path(&env, "list");

    env.but("agent list")
        .env("BUT_AGENT_REGISTRY_PATH", &registry_path)
        .assert()
        .success()
        .stdout_eq(snapshot!("snapshots/agent/list-empty.stdout"));

    env.but("agent register --pid 12345 --start-time 1730000000 --as rust-implementer --ttl 1h")
        .env("BUT_AGENT_REGISTRY_PATH", &registry_path)
        .assert()
        .success()
        .stdout_eq(snapshot!("snapshots/agent/register-known.stdout"));

    env.but("agent list")
        .env("BUT_AGENT_REGISTRY_PATH", &registry_path)
        .assert()
        .success()
        .stdout_eq(snapshot!("snapshots/agent/list-populated.stdout"));

    Ok(())
}

#[test]
fn agent_unregister_known_and_unknown_pid_is_idempotent() -> anyhow::Result<()> {
    let env = ident_fixture("ident-agent-unregister", RUST_IMPLEMENTER_AGENT)?;
    let registry_path = registry_path(&env, "unregister");

    env.but("agent register --pid 12345 --start-time 1730000000 --as rust-implementer --ttl 1h")
        .env("BUT_AGENT_REGISTRY_PATH", &registry_path)
        .assert()
        .success()
        .stdout_eq(snapshot!("snapshots/agent/register-known.stdout"));

    env.but("agent unregister --pid 12345 --start-time 1730000000")
        .env("BUT_AGENT_REGISTRY_PATH", &registry_path)
        .assert()
        .success()
        .stdout_eq(snapshot!("snapshots/agent/unregister-known.stdout"));

    env.but("agent list")
        .env("BUT_AGENT_REGISTRY_PATH", &registry_path)
        .assert()
        .success()
        .stdout_eq(snapshot!("snapshots/agent/list-empty.stdout"));

    env.but("agent unregister --pid 99999")
        .env("BUT_AGENT_REGISTRY_PATH", &registry_path)
        .assert()
        .success()
        .stdout_eq(snapshot!("snapshots/agent/unregister-unknown.stdout"));

    Ok(())
}

#[test]
fn agent_whoami_without_registration_reports_current_process_key() -> anyhow::Result<()> {
    let env = ident_fixture("ident-agent-whoami-missing", RUST_IMPLEMENTER_AGENT)?;
    let registry_path = registry_path(&env, "whoami-missing");

    env.but("agent whoami")
        .env("BUT_AGENT_REGISTRY_PATH", &registry_path)
        .assert()
        .failure()
        .stdout_eq(snapbox::str![])
        .stderr_eq(snapbox::str![[r#"
Error: no agent registration for pid [..] start_time [..]

"#]]);

    Ok(())
}

#[test]
fn agent_migrate_writes_agents_toml() -> anyhow::Result<()> {
    let env = permissions_only_fixture(LEGACY_PERMISSIONS_TOML)?;
    let permissions_path = governance_path(&env, "permissions.toml");
    let agents_path = governance_path(&env, "agents.toml");

    let output = env.but("agent migrate").output()?;
    assert!(
        output.status.success(),
        "agent migrate must succeed for a permissions.toml-only working tree; stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(AGENT_MIGRATE_REF_PIN_CAVEAT),
        "agent migrate stdout must include the ref-pin caveat naming the add+delete commit step, got: {stdout}"
    );
    assert!(
        permissions_path.exists(),
        "agent migrate must leave the legacy permissions.toml in place for the operator to delete"
    );
    assert_eq!(
        fs::read_to_string(&agents_path)?,
        MIGRATED_AGENTS_TOML,
        "agent migrate must only rename [[principal]] table headers to [[agent]]"
    );

    Ok(())
}

#[test]
fn agent_migrate_idempotent() -> anyhow::Result<()> {
    let env = permissions_only_fixture(LEGACY_PERMISSIONS_TOML)?;
    let agents_path = governance_path(&env, "agents.toml");
    fs::write(&agents_path, MIGRATED_AGENTS_TOML)?;

    let output = env.but("agent migrate").output()?;
    assert!(
        output.status.success(),
        "agent migrate must succeed when agents.toml already exists; stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("already migrated") && stdout.contains("no change"),
        "idempotent agent migrate stdout must report the no-op distinctly, got: {stdout}"
    );
    assert_eq!(
        fs::read_to_string(&agents_path)?,
        MIGRATED_AGENTS_TOML,
        "idempotent agent migrate must not rewrite an existing non-empty agents.toml"
    );

    Ok(())
}

#[test]
fn agent_migrate_missing_source() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack")?;
    let agents_path = governance_path(&env, "agents.toml");

    let output = env.but("agent migrate").output()?;
    assert!(
        !output.status.success(),
        "agent migrate must fail without permissions.toml or agents.toml"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(".gitbutler/permissions.toml"),
        "missing-source stderr must name .gitbutler/permissions.toml, got: {stderr}"
    );
    assert!(
        !agents_path.exists(),
        "missing-source agent migrate must not create agents.toml"
    );

    Ok(())
}

fn ident_fixture(_name: &str, agents_toml: &str) -> anyhow::Result<Sandbox> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack")?;
    env.invoke_bash(format!(
        r#"
base=$(git rev-parse refs/heads/main)
index=$(mktemp)
export GIT_INDEX_FILE="$index"
git read-tree "$base"
agents_blob=$(git hash-object -w --stdin <<'EOF'
{agents_toml}
EOF
)
git update-index --add --cacheinfo 100644 "$agents_blob" .gitbutler/agents.toml
tree=$(git write-tree)
commit=$(printf 'seed agents\n' | git commit-tree "$tree" -p "$base")
git update-ref refs/heads/main "$commit"
rm "$index"
unset GIT_INDEX_FILE
"#
    ));
    Ok(env)
}

fn permissions_only_fixture(permissions_toml: &str) -> anyhow::Result<Sandbox> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack")?;
    let gitbutler_dir = env.projects_root().join(".gitbutler");
    fs::create_dir_all(&gitbutler_dir)?;
    fs::write(gitbutler_dir.join("permissions.toml"), permissions_toml)?;
    Ok(env)
}

fn governance_path(env: &Sandbox, name: &str) -> PathBuf {
    env.projects_root().join(".gitbutler").join(name)
}

fn registry_path(env: &Sandbox, name: &str) -> PathBuf {
    env.projects_root().join(format!("{name}-runtime.toml"))
}
