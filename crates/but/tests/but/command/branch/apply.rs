use bstr::ByteSlice;
use snapbox::str;

use crate::command::util;
use crate::utils::{CommandExt, Sandbox};

#[cfg(not(feature = "legacy"))]
#[test]
fn single_branch() -> anyhow::Result<()> {
    let env = Sandbox::open_with_default_settings("one-fork")?;
    insta::assert_snapshot!(env.git_log()?, @r"
    * bf53300 (A) add A
    | * b1540e5 (HEAD -> main) M
    |/  
    | * 0e391b2 (origin/B) add B
    |/  
    * e31e6ca (origin/main, origin/HEAD) add init
    ");

    env.but("apply A")
        .assert()
        .success()
        .stderr_eq(str![])
        .stdout_eq(str![[r#"
Applied branch 'main' to workspace
Applied branch 'A' to workspace

"#]]);

    insta::assert_snapshot!(env.workspace_debug_at_head()?, @r"
    📕🏘️:0:gitbutler/workspace[🌳] <> ✓! on e31e6ca
    ├── ≡📙:2:A on e31e6ca {1}
    │   └── 📙:2:A
    │       └── ·bf53300 (🏘️)
    └── ≡📙:1:main on e31e6ca {2}
        └── 📙:1:main
            └── ·b1540e5 (🏘️)
    ");

    insta::assert_snapshot!(env.git_log()?, @r"
    *   d87b903 (HEAD -> gitbutler/workspace) GitButler Workspace Commit
    |\  
    | * bf53300 (A) add A
    * | b1540e5 (main) M
    |/  
    | * 0e391b2 (origin/B) add B
    |/  
    * e31e6ca (origin/main, origin/HEAD) add init
    ");

    env.but("apply origin/B")
        .assert()
        .success()
        .stdout_eq(str![[r#"
Applied remote branch 'origin/B' to workspace

"#]])
        .stderr_eq(str![""]);
    insta::assert_snapshot!(env.workspace_debug_at_head()?, @r"
    📕🏘️:0:gitbutler/workspace[🌳] <> ✓! on e31e6ca
    ├── ≡📙:3:B <> origin/B →:4: on e31e6ca {1}
    │   └── 📙:3:B <> origin/B →:4:
    │       └── ❄️0e391b2 (🏘️)
    ├── ≡📙:2:A on e31e6ca {2}
    │   └── 📙:2:A
    │       └── ·bf53300 (🏘️)
    └── ≡📙:1:main on e31e6ca {3}
        └── 📙:1:main
            └── ·b1540e5 (🏘️)
    ");

    // TODO: should be success and create a local tracking branch.
    insta::assert_snapshot!(env.git_log()?, @r"
    *-.   7bcf528 (HEAD -> gitbutler/workspace) GitButler Workspace Commit
    |\ \  
    | | * 0e391b2 (origin/B, B) add B
    | * | bf53300 (A) add A
    | |/  
    * / b1540e5 (main) M
    |/  
    * e31e6ca (origin/main, origin/HEAD) add init
    ");
    Ok(())
}

use utils::create_local_branch_with_commit;

use crate::command::branch::apply::utils::create_local_branch_with_commit_with_message;

#[test]
#[serial_test::serial]
fn branch_apply_readonly_denied() -> anyhow::Result<()> {
    let env = governed_apply_env()?;
    let repo = env.open_repo()?;
    let target_before = ref_id(&repo, "refs/remotes/origin/main")?;
    let workspace_before = ref_id(&repo, "refs/heads/gitbutler/workspace")?;

    let output = env
        .but("--format json apply feature-branch")
        .allow_json()
        .env("BUT_AUTHZ_ALLOW_ENV_HANDLE", "1")
        .env("BUT_AGENT_HANDLE", "ro")
        .output()?;
    assert_cli_denial(
        &output,
        "perm.denied",
        "contents:write",
        "read-only branch apply must be denied by the commit gate",
    );
    assert_eq!(
        ref_id(&repo, "refs/remotes/origin/main")?,
        target_before,
        "read-only denial must leave the workspace target ref unchanged"
    );
    assert_eq!(
        ref_id(&repo, "refs/heads/gitbutler/workspace")?,
        workspace_before,
        "read-only denial must happen before branch apply moves the workspace ref"
    );

    let dev_env = governed_apply_env()?;
    let dev_output = dev_env
        .but("--format json apply feature-branch")
        .allow_json()
        .env("BUT_AUTHZ_ALLOW_ENV_HANDLE", "1")
        .env("BUT_AGENT_HANDLE", "dev")
        .output()?;
    assert!(
        dev_output.status.success(),
        "contents:write principal should proceed through branch apply; stdout: {}; stderr: {}",
        String::from_utf8_lossy(&dev_output.stdout),
        String::from_utf8_lossy(&dev_output.stderr)
    );
    assert_no_governance_denial(&dev_output, "contents:write branch apply");

    Ok(())
}

#[test]
fn local_branch() -> anyhow::Result<()> {
    let env = Sandbox::open_or_init_scenario_with_target_and_default_settings("one-stack")?;
    insta::assert_snapshot!(env.git_log()?, @"
    * edd3eb7 (HEAD -> gitbutler/workspace) GitButler Workspace Commit
    * 9477ae7 (A) add A
    * 0dc3733 (origin/main, origin/HEAD, main) add M
    ");

    env.setup_metadata(&["A"])?;

    let branch_name = "feature-branch";
    create_local_branch_with_commit(&env, branch_name);

    // Apply the local branch
    env.but("apply")
        .arg(branch_name)
        .assert()
        .success()
        .stderr_eq(str![])
        .stdout_eq(str![[r#"
Applied branch 'feature-branch' to workspace

"#]]);
    // It's idempotent and can produce a shell value.
    env.but("--format shell apply feature-branch")
        .allow_json()
        .assert()
        .success()
        .stderr_eq(str![])
        .stdout_eq(str![[r#"
refs/heads/feature-branch

"#]]);

    // It actually applied the branch, by merging it in.
    insta::assert_snapshot!(env.git_log()?, @r"
    *   9d5d9e5 (HEAD -> gitbutler/workspace) GitButler Workspace Commit
    |\  
    | * 9f9d5a6 (feature-branch) Add feature
    * | 9477ae7 (A) add A
    |/  
    * 0dc3733 (origin/main, origin/HEAD, main, gitbutler/target) add M
    ");

    Ok(())
}

#[test]
fn local_branch_with_json_output() -> anyhow::Result<()> {
    let env = Sandbox::open_or_init_scenario_with_target_and_default_settings("one-stack")?;
    insta::assert_snapshot!(env.git_log()?, @"
    * edd3eb7 (HEAD -> gitbutler/workspace) GitButler Workspace Commit
    * 9477ae7 (A) add A
    * 0dc3733 (origin/main, origin/HEAD, main) add M
    ");

    env.setup_metadata(&["A"])?;

    create_local_branch_with_commit(&env, "feature-branch");

    // Apply with JSON output
    env.but("--format json apply feature-branch")
        .allow_json()
        .assert()
        .success()
        .stdout_eq(str![[r#"
{
  "name": {
    "full": "refs/heads/feature-branch",
    "full_bytes": [
      114,
      101,
      102,
      115,
      47,
      104,
      101,
      97,
      100,
      115,
      47,
      102,
      101,
      97,
      116,
      117,
      114,
      101,
      45,
      98,
      114,
      97,
      110,
      99,
      104
    ]
  },
  "target_id": "9f9d5a694afe171f5f9c72f8cf06db6210c3cf43",
  "target_ref": null
}

"#]])
        .stderr_eq(str![]);

    insta::assert_snapshot!(env.git_log()?, @r"
    *   9d5d9e5 (HEAD -> gitbutler/workspace) GitButler Workspace Commit
    |\  
    | * 9f9d5a6 (feature-branch) Add feature
    * | 9477ae7 (A) add A
    |/  
    * 0dc3733 (origin/main, origin/HEAD, main, gitbutler/target) add M
    ");

    Ok(())
}

#[test]
fn remote_branch_creates_local_tracking_branch_automatically() -> anyhow::Result<()> {
    let env = Sandbox::open_or_init_scenario_with_target_and_default_settings("one-stack")?;
    insta::assert_snapshot!(env.git_log()?, @"
    * edd3eb7 (HEAD -> gitbutler/workspace) GitButler Workspace Commit
    * 9477ae7 (A) add A
    * 0dc3733 (origin/main, origin/HEAD, main) add M
    ");

    env.setup_metadata(&["A"])?;

    // Create a remote branch reference
    env.invoke_bash(
        r#"
    git checkout origin/main
    git commit -m 'Add remote feature' --allow-empty
    git update-ref refs/remotes/origin/remote-feature HEAD
    git checkout gitbutler/workspace
"#,
    );

    // Apply the remote branch, by its shortest name only.
    env.but("apply origin/remote-feature")
        .assert()
        .success()
        .stderr_eq(str![])
        .stdout_eq(str![[r#"
Applied remote branch 'origin/remote-feature' to workspace

"#]]);

    // It created a local tracking branch.
    insta::assert_snapshot!(env.git_log()?, @r"
    *   1bb7daf (HEAD -> gitbutler/workspace) GitButler Workspace Commit
    |\  
    | * ba02e5f (origin/remote-feature, remote-feature) Add remote feature
    * | 9477ae7 (A) add A
    |/  
    * 0dc3733 (origin/main, origin/HEAD, main, gitbutler/target) add M
    ");

    Ok(())
}

#[test]
fn remote_branch_short_name_resolves_to_unique_remote_tracking_branch() -> anyhow::Result<()> {
    let env = Sandbox::open_or_init_scenario_with_target_and_default_settings("one-stack")?;
    insta::assert_snapshot!(env.git_log()?, @"
    * edd3eb7 (HEAD -> gitbutler/workspace) GitButler Workspace Commit
    * 9477ae7 (A) add A
    * 0dc3733 (origin/main, origin/HEAD, main) add M
    ");

    env.setup_metadata(&["A"])?;

    // Create a remote-only branch reference.
    env.invoke_bash(
        r#"
    git checkout origin/main
    git commit -m 'Add remote feature' --allow-empty
    git update-ref refs/remotes/origin/remote-feature HEAD
    git checkout gitbutler/workspace
"#,
    );

    // Apply the remote branch by its bare name.
    env.but("apply remote-feature")
        .assert()
        .success()
        .stderr_eq(str![])
        .stdout_eq(str![[r#"
Applied remote branch 'origin/remote-feature' to workspace

"#]]);

    // It created the same local tracking branch as the qualified form.
    insta::assert_snapshot!(env.git_log()?, @r"
    *   1bb7daf (HEAD -> gitbutler/workspace) GitButler Workspace Commit
    |\  
    | * ba02e5f (origin/remote-feature, remote-feature) Add remote feature
    * | 9477ae7 (A) add A
    |/  
    * 0dc3733 (origin/main, origin/HEAD, main, gitbutler/target) add M
    ");

    Ok(())
}

#[test]
fn remote_branch_short_name_requires_disambiguation_across_multiple_remotes() -> anyhow::Result<()>
{
    let env = Sandbox::open_or_init_scenario_with_target_and_default_settings("one-stack")?;
    insta::assert_snapshot!(env.git_log()?, @"
    * edd3eb7 (HEAD -> gitbutler/workspace) GitButler Workspace Commit
    * 9477ae7 (A) add A
    * 0dc3733 (origin/main, origin/HEAD, main) add M
    ");

    env.setup_metadata(&["A"])?;

    // Create two configured remotes that both expose the same short branch name.
    env.invoke_bash(
        r#"
    git remote add upstream .
    git checkout origin/main
    git commit -m 'Add remote feature' --allow-empty
    git update-ref refs/remotes/origin/remote-feature HEAD
    git update-ref refs/remotes/upstream/remote-feature HEAD
    git checkout gitbutler/workspace
"#,
    );

    env.but("apply remote-feature")
        .assert()
        .failure()
        .stderr_eq(str![[r#"
Failed to apply branch. The reference 'remote-feature' did not exist

"#]])
        .stdout_eq(str![""]);

    env.but("apply origin/remote-feature")
        .assert()
        .success()
        .stderr_eq(str![])
        .stdout_eq(str![[r#"
Applied remote branch 'origin/remote-feature' to workspace

"#]]);

    insta::assert_snapshot!(env.git_log()?, @r"
    *   1bb7daf (HEAD -> gitbutler/workspace) GitButler Workspace Commit
    |\  
    | * ba02e5f (upstream/remote-feature, origin/remote-feature, remote-feature) Add remote feature
    * | 9477ae7 (A) add A
    |/  
    * 0dc3733 (origin/main, origin/HEAD, main, gitbutler/target) add M
    ");

    Ok(())
}

#[test]
fn concurrent_apply_of_independent_branches_succeeds() -> anyhow::Result<()> {
    let env = Sandbox::open_or_init_scenario_with_target_and_default_settings("one-stack")?;
    env.setup_metadata(&["A"])?;

    create_local_branch_with_commit(&env, "feature-branch-a");
    create_local_branch_with_commit_with_message(&env, "feature-branch-b", "Add other feature");

    let child_a = util::but_std_cmd(&env, "apply feature-branch-a").spawn()?;
    let child_b = util::but_std_cmd(&env, "apply feature-branch-b").spawn()?;

    let out_a = child_a.wait_with_output()?;
    let out_b = child_b.wait_with_output()?;

    assert!(
        out_a.status.success(),
        "apply feature-branch-a failed: {}",
        out_a.stderr.as_bstr()
    );
    assert!(
        out_b.status.success(),
        "apply feature-branch-b failed: {}",
        out_b.stderr.as_bstr()
    );

    let status = util::status_json(&env)?;
    util::find_branch(&status, "feature-branch-a")?;
    util::find_branch(&status, "feature-branch-b")?;

    Ok(())
}

#[test]
fn nonexistent_branch() -> anyhow::Result<()> {
    let env = Sandbox::open_or_init_scenario_with_target_and_default_settings("one-stack")?;

    // Try to apply a branch that doesn't exist
    env.but("apply nonexistent-branch")
        .assert()
        .failure()
        .stderr_eq(str![[r#"
Failed to apply branch. The reference 'nonexistent-branch' did not exist

"#]])
        .stdout_eq(str![""]);

    Ok(())
}

#[test]
fn nonexistent_branch_with_json() -> anyhow::Result<()> {
    let env = Sandbox::open_or_init_scenario_with_target_and_default_settings("one-stack")?;

    // Try to apply a branch that doesn't exist with JSON output
    env.but("--format json apply nonexistent-branch")
        .allow_json()
        .assert()
        .failure()
        .stderr_eq(str![[r#"
Failed to apply branch. The reference 'nonexistent-branch' did not exist

"#]]);
    // Note: Currently the apply function doesn't output anything with JSON when branch not found
    // This might be improved to output an error in JSON format

    Ok(())
}

#[test]
fn multiple_branches_sequentially() -> anyhow::Result<()> {
    let env = Sandbox::open_or_init_scenario_with_target_and_default_settings("one-stack")?;
    insta::assert_snapshot!(env.git_log()?, @"
    * edd3eb7 (HEAD -> gitbutler/workspace) GitButler Workspace Commit
    * 9477ae7 (A) add A
    * 0dc3733 (origin/main, origin/HEAD, main) add M
    ");

    env.setup_metadata(&["A"])?;

    let f1 = "feature-1";
    create_local_branch_with_commit_with_message(&env, f1, "Add feature 1");
    let f2 = "feature-2";
    create_local_branch_with_commit_with_message(&env, f2, "Add feature 2");

    // Apply both branches
    env.but("apply")
        .arg(f1)
        .assert()
        .success()
        .stdout_eq(str![[r#"
Applied branch 'feature-1' to workspace

"#]])
        .stderr_eq(str![]);

    env.but("apply")
        .arg(f2)
        .assert()
        .success()
        .stdout_eq(str![[r#"
Applied branch 'feature-2' to workspace

"#]])
        .stderr_eq(str![]);

    insta::assert_snapshot!(env.git_log()?, @r"
    *-.   7044ae9 (HEAD -> gitbutler/workspace) GitButler Workspace Commit
    |\ \  
    | | * 4e81b31 (feature-2) Add feature 2
    | * | 9c2fe5c (feature-1) Add feature 1
    | |/  
    * / 9477ae7 (A) add A
    |/  
    * 0dc3733 (origin/main, origin/HEAD, main, gitbutler/target) add M
    ");
    Ok(())
}

#[test]
fn apply_branch_conflicting_with_workspace_reports_error() -> anyhow::Result<()> {
    let env = Sandbox::open_or_init_scenario_with_target_and_default_settings("one-stack")?;
    insta::assert_snapshot!(env.git_log()?, @"
    * edd3eb7 (HEAD -> gitbutler/workspace) GitButler Workspace Commit
    * 9477ae7 (A) add A
    * 0dc3733 (origin/main, origin/HEAD, main) add M
    ");

    env.setup_metadata(&["A"])?;

    env.invoke_bash(
        r#"
    git checkout main -b conflicting-branch;
    echo 'conflicting-A-content' > A;
    git add A;
    git commit -m 'Add conflicting A';
    git checkout gitbutler/workspace;
    "#,
    );

    // It's notable that this behaviour is different from what the GUI does, which
    // unapplies all conflicting instead.
    env.but("apply conflicting-branch")
        .assert()
        .failure()
        .stderr_eq(str![[r#"
Failed to apply branch. 'conflicting-branch' conflicts with existing stack in the workspace: A

"#]])
        .stdout_eq(str![""]);

    Ok(())
}

fn governed_apply_env() -> anyhow::Result<Sandbox> {
    let env = Sandbox::open_or_init_scenario_with_target_and_default_settings("one-stack")?;
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
name = "feature-branch"
protected = false
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
write_governance_commit refs/remotes/origin/main
"#,
    );
    env.setup_metadata(&["A"])?;
    create_local_branch_with_commit(&env, "feature-branch");
    Ok(env)
}

fn ref_id(repo: &gix::Repository, ref_name: &str) -> anyhow::Result<gix::ObjectId> {
    Ok(repo.find_reference(ref_name)?.peel_to_id()?.detach())
}

fn assert_cli_denial(
    output: &std::process::Output,
    code: &str,
    expected_message_text: &str,
    reason: &str,
) {
    assert_eq!(
        output.status.code(),
        Some(1),
        "{reason}; denial must exit with code 1. stdout: {}; stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let envelope = parse_cli_error_envelope(output, reason);
    assert_eq!(
        envelope.code, code,
        "{reason}; expected exact denial code {code}"
    );
    assert!(
        envelope.message.contains(expected_message_text),
        "{reason}; expected error.message to contain {expected_message_text:?}, got: {}",
        envelope.message
    );
}

fn assert_no_governance_denial(output: &std::process::Output, label: &str) {
    assert!(
        output.status.code() != Some(1) || parse_cli_error_envelope_opt(output).is_none(),
        "{label} must not return a structured governance denial envelope: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains(r#""code":"perm.denied""#)
            && !stderr.contains(r#""code":"branch.protected""#)
            && !stderr.contains(r#""code":"config.invalid""#),
        "{label} must not fail with a governance denial: {stderr}"
    );
}

#[derive(Debug, Clone)]
struct CliErrorEnvelope {
    code: String,
    message: String,
}

fn parse_cli_error_envelope(output: &std::process::Output, reason: &str) -> CliErrorEnvelope {
    parse_cli_error_envelope_opt(output).unwrap_or_else(|| {
        panic!(
            "{reason}; stderr must contain a parseable CLI JSON error envelope, got: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn parse_cli_error_envelope_opt(output: &std::process::Output) -> Option<CliErrorEnvelope> {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let json = stderr.lines().find_map(json_object_from_line)?;
    let value = serde_json::from_str::<serde_json::Value>(json).ok()?;
    let error = value.get("error")?;
    let code = error.get("code")?.as_str()?.to_owned();
    let message = error.get("message")?.as_str()?.to_owned();
    Some(CliErrorEnvelope { code, message })
}

fn json_object_from_line(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    let start = trimmed.find('{')?;
    let end = trimmed.rfind('}')?;
    (start <= end).then_some(&trimmed[start..=end])
}

mod utils {
    use crate::utils::Sandbox;

    pub fn create_local_branch_with_commit(env: &Sandbox, name: &str) {
        create_local_branch_with_commit_with_message(env, name, "Add feature")
    }

    pub fn create_local_branch_with_commit_with_message(
        env: &Sandbox,
        name: &str,
        commit_message: &str,
    ) {
        env.invoke_bash(format!(
            r#"
    git checkout main -b {name};
    git commit -m '{commit_message}' --allow-empty;
    git checkout gitbutler/workspace;
        "#
        ));
    }
}
