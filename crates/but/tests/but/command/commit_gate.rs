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
warning: .gitbutler/permissions.toml is deprecated; run: but agent migrate
Error: {"error":{"code":"branch.protected","message":"direct commits to protected branch /"A/" are denied for principal /"dev/"; land changes through a reviewed merge","remediation_hint":"open a reviewed merge into A instead of committing directly","class":"actor_correctable","held_permissions":["contents:write"],"authorized_actions":[{"command":"but commit","effect":"create a commit on an unprotected feature branch ref"},{"command":"but perm list","effect":"list your own effective permissions (self-discovery)"}]}}

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
warning: .gitbutler/permissions.toml is deprecated; run: but agent migrate
Error: {"error":{"code":"branch.protected","message":"direct commits to protected branch /"A/" are denied for principal /"dev/"; land changes through a reviewed merge","remediation_hint":"open a reviewed merge into A instead of committing directly","class":"actor_correctable","held_permissions":["contents:write"],"authorized_actions":[{"command":"but commit","effect":"create a commit on an unprotected feature branch ref"},{"command":"but perm list","effect":"list your own effective permissions (self-discovery)"}]}}

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
warning: .gitbutler/permissions.toml is deprecated; run: but agent migrate
Error: {"error":{"code":"perm.denied","message":"action requires contents:write; authorization denied (held permissions: contents:read)","remediation_hint":"request a reviewed merge or ask a maintainer to grant contents:write","class":"actor_correctable","held_permissions":["contents:read"],"authorized_actions":[{"command":"but perm list","effect":"list your own effective permissions (self-discovery)"}]}}

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
warning: .gitbutler/permissions.toml is deprecated; run: but agent migrate
Error: {"error":{"code":"config.invalid","message":"invalid governance config: parsing .gitbutler/gates.toml at refs/heads/A","class":"operator_required","held_permissions":[],"authorized_actions":[],"do_not":"do not retry — an operator must fix the committed .gitbutler config"}}

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
