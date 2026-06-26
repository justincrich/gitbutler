use std::path::PathBuf;

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

fn registry_path(env: &Sandbox, name: &str) -> PathBuf {
    env.projects_root().join(format!("{name}-runtime.toml"))
}
