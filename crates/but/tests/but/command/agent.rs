use std::{fs, path::PathBuf};

use crate::utils::Sandbox;

macro_rules! snapshot {
    ($path:literal) => {
        include_str!($path)
    };
}

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

const GOVERNANCE_GATES_TOML: &str = r#"[[branch]]
name = "main"
protected = true
"#;

const PREEXISTING_AGENTS_TOML_SENTINEL: &str = r#"# sentinel: hand-authored agents.toml must not be regenerated
[[agent]]
id = "hand-authored"
permissions = ["contents:read"]
preserve_me = "do-not-overwrite"
"#;

const AGENT_MIGRATE_REF_PIN_CAVEAT: &str = "Commit the add of .gitbutler/agents.toml and the delete of .gitbutler/permissions.toml together.";

#[test]
fn agent_help_lists_verbs() -> anyhow::Result<()> {
    let env = Sandbox::empty()?;

    env.but("agent --help")
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"Manage runtime agent registrations

Usage: but agent [OPTIONS] [COMMAND]

Commands:
  list     List the committed agent roster from .gitbutler/agents.toml
  migrate  Rewrite working-tree .gitbutler/permissions.toml to .gitbutler/agents.toml

Options:
      --format <FORMAT>
          Explicitly control how output should be formatted.
[..]
          If unset and from a terminal, it defaults to human output, when redirected it's for
          shells.

          Possible values:
          - human: The output to write is supposed to be for human consumption, and can be more
            verbose
          - agent: The output is for an AI coding agent, rendered as human-readable text
          - shell: The output should be suitable for shells, and assigning the major result to
            variables so that it can be reused in subsequent CLI invocations
          - json:  Output detailed information as JSON for tool consumption
          - none:  Do not output anything, like redirecting to /dev/null
[..]
          [env: BUT_OUTPUT_FORMAT=]
          [default: human]

  -h, --help
          Print help (see a summary with '-h')

"#]]);

    Ok(())
}

#[test]
fn agent_list_committed_prints_roster() -> anyhow::Result<()> {
    // The runtime registry was removed (identity is env-primary). `but agent
    // list` reports the committed roster from `.gitbutler/agents.toml`.
    let env = ident_fixture("ident-agent-list-committed", RUST_IMPLEMENTER_AND_REVIEWER_AGENTS)?;

    env.but("agent list --committed")
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
rust-implementer
rust-reviewer

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
    fs::write(&agents_path, PREEXISTING_AGENTS_TOML_SENTINEL)?;

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
        PREEXISTING_AGENTS_TOML_SENTINEL,
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

#[test]
#[serial_test::serial]
fn agent_migrate_initial_writes_agents_toml_with_caveat() -> anyhow::Result<()> {
    let env = permissions_only_committed_fixture(LEGACY_PERMISSIONS_TOML)?;
    let agents_path = governance_path(&env, "agents.toml");

    env.but("agent migrate")
        .assert()
        .success()
        .stdout_eq(snapshot!("snapshots/agent/migrate-initial.stdout"))
        .stderr_eq(snapbox::str![]);

    assert!(
        agents_path.exists(),
        "agent migrate must write .gitbutler/agents.toml into the working tree"
    );

    Ok(())
}

#[test]
#[serial_test::serial]
fn agent_migrate_idempotent_rerun_is_noop() -> anyhow::Result<()> {
    let env = permissions_only_committed_fixture(LEGACY_PERMISSIONS_TOML)?;
    let agents_path = governance_path(&env, "agents.toml");

    env.but("agent migrate").assert().success();
    let agents_before = fs::read(&agents_path)?;

    env.but("agent migrate")
        .assert()
        .success()
        .stdout_eq(snapshot!("snapshots/agent/migrate-idempotent.stdout"))
        .stderr_eq(snapbox::str![]);

    assert_eq!(
        fs::read(&agents_path)?,
        agents_before,
        "idempotent agent migrate must leave .gitbutler/agents.toml byte-unchanged"
    );

    Ok(())
}

#[test]
#[serial_test::serial]
fn agent_migrate_permissions_only_emits_deprecation_warning() -> anyhow::Result<()> {
    let env = permissions_only_committed_fixture(LEGACY_PERMISSIONS_TOML)?;

    env.but("perm list --principal rust-implementer")
        .env("BUT_AGENT_HANDLE", "rust-implementer")
        .env("BUT_AUTHZ_ALLOW_ENV_HANDLE", "1")
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
rust-implementer:
  contents:write

"#]])
        .stderr_eq(snapshot!(
            "snapshots/agent/permissions-only-deprecation.stderr"
        ));

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

fn permissions_only_committed_fixture(permissions_toml: &str) -> anyhow::Result<Sandbox> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack")?;
    env.invoke_bash(format!(
        r#"
base=$(git rev-parse refs/heads/main)
index=$(mktemp)
export GIT_INDEX_FILE="$index"
git read-tree "$base"
permissions_blob=$(git hash-object -w --stdin <<'EOF'
{permissions_toml}
EOF
)
gates_blob=$(git hash-object -w --stdin <<'EOF'
{GOVERNANCE_GATES_TOML}
EOF
)
git update-index --add --cacheinfo 100644 "$permissions_blob" .gitbutler/permissions.toml
git update-index --add --cacheinfo 100644 "$gates_blob" .gitbutler/gates.toml
tree=$(git write-tree)
commit=$(printf 'seed legacy permissions\n' | git commit-tree "$tree" -p "$base")
git update-ref refs/heads/main "$commit"
rm "$index"
unset GIT_INDEX_FILE
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
{permissions_toml}
EOF
cat >.gitbutler/gates.toml <<'EOF'
{GOVERNANCE_GATES_TOML}
EOF
"#
    ));
    Ok(env)
}

fn governance_path(env: &Sandbox, name: &str) -> PathBuf {
    env.projects_root().join(".gitbutler").join(name)
}
