use crate::utils::{CommandExt as _, Sandbox};

#[test]
fn commit_gate_denies_protected_branch() -> anyhow::Result<()> {
    let env = governed_env("one-stack", Some("A"))?;

    env.file("protected.txt", "protected");
    env.but("--format json commit2 -m protected")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "dev")
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
        .env("BUT_AGENT_HANDLE", "ro")
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
        .env("BUT_AGENT_HANDLE", "dev")
        .assert()
        .failure()
        .stdout_eq(snapbox::str![[r#"
"#]])
        .stderr_eq(snapbox::str![[r#"
Error: {"error":{"code":"config.invalid","message":"invalid governance config: parsing .gitbutler/gates.toml at refs/heads/A[..]"}}

"#]]);

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
