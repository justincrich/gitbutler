use snapbox::str;

use crate::command::util;
use crate::utils::{CommandExt, Sandbox};

fn pretty_status(env: &Sandbox) -> anyhow::Result<String> {
    Ok(serde_json::to_string_pretty(&util::status_json(env)?)?)
}

fn raw_json_status(env: &Sandbox) -> anyhow::Result<String> {
    let output = env.but("--format json status").allow_json().output()?;
    Ok(format!(
        "status={}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}

fn install_editor_script(env: &Sandbox, script: &str) -> anyhow::Result<()> {
    env.file("editor.sh", script);
    Ok(())
}

#[test]
fn integrate_pull_rebase_applies_and_snapshots_before_and_after() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("branch-integrate-diverged")?;

    insta::assert_snapshot!(env.git_log()?, @r"
    *   a952a0b (HEAD -> gitbutler/workspace) GitButler Workspace Commit
    |\  
    | * 643ade3 (A) add only-on-local
    |/  
    | * 28baf9a (origin/A) add only-on-remote
    |/  
    * 0dc3733 (origin/main, origin/HEAD, main) add M
    ");
    insta::assert_snapshot!(pretty_status(&env)?, @r#"
    {
      "unassignedChanges": [],
      "stacks": [],
      "mergeBase": {
        "cliId": "",
        "commitId": "0dc37334a458df421bf67ea806103bf5004845dd",
        "createdAt": "2000-01-01T00:00:00+00:00",
        "message": "add M\n",
        "authorName": "author",
        "authorEmail": "author@example.com",
        "conflicted": null,
        "reviewId": null,
        "changes": null
      },
      "upstreamState": {
        "behind": 0,
        "latestCommit": {
          "cliId": "",
          "commitId": "0dc37334a458df421bf67ea806103bf5004845dd",
          "createdAt": "2000-01-01T00:00:00+00:00",
          "message": "add M\n",
          "authorName": "author",
          "authorEmail": "author@example.com",
          "conflicted": null,
          "reviewId": null,
          "changes": null
        },
        "lastFetched": null
      }
    }
    "#);

    env.but("branch update A")
        .assert()
        .success()
        .stderr_eq(str![])
        .stdout_eq(str![[r#"
Updated branch A.

"#]]);

    insta::assert_snapshot!(env.git_log()?, @r"
    *   6a3496e (HEAD -> gitbutler/workspace) GitButler Workspace Commit
    |\  
    | * 74faa12 (A) add only-on-local
    | * 28baf9a (origin/A) add only-on-remote
    |/  
    * 0dc3733 (origin/main, origin/HEAD, main, gitbutler/target) add M
    ");
    insta::assert_snapshot!(pretty_status(&env)?, @r#"
    {
      "unassignedChanges": [],
      "stacks": [],
      "mergeBase": {
        "cliId": "",
        "commitId": "0dc37334a458df421bf67ea806103bf5004845dd",
        "createdAt": "2000-01-01T00:00:00+00:00",
        "message": "add M\n",
        "authorName": "author",
        "authorEmail": "author@example.com",
        "conflicted": null,
        "reviewId": null,
        "changes": null
      },
      "upstreamState": {
        "behind": 0,
        "latestCommit": {
          "cliId": "",
          "commitId": "0dc37334a458df421bf67ea806103bf5004845dd",
          "createdAt": "2000-01-01T00:00:00+00:00",
          "message": "add M\n",
          "authorName": "author",
          "authorEmail": "author@example.com",
          "conflicted": null,
          "reviewId": null,
          "changes": null
        },
        "lastFetched": null
      }
    }
    "#);

    Ok(())
}

#[test]
fn integrate_smart_squash_applies_matching_change_ids() -> anyhow::Result<()> {
    let env =
        Sandbox::init_scenario_with_target_and_default_settings("branch-integrate-smart-squash")?;

    insta::assert_snapshot!(env.git_log()?, @"
    * 2662ee8 (HEAD -> gitbutler/workspace, A) add only-on-local
    | * c42227a (origin/A) add only-on-remote
    |/  
    * 0dc3733 (origin/main, origin/HEAD, main) add M
    ");
    insta::assert_snapshot!(raw_json_status(&env)?, @r"
    status=exit status: 1
    stdout:

    stderr:
    Error: GitButler mode exit required: please run `but teardown` to preserve your work.
    ");

    env.but("branch update A --strategy smart-squash")
        .assert()
        .success()
        .stderr_eq(str![])
        .stdout_eq(str![[r#"
Updated branch A.

"#]]);

    insta::assert_snapshot!(env.git_log()?, @"
    * bf02b24 (HEAD -> gitbutler/workspace, A) add only-on-remote
    | * c42227a (origin/A) add only-on-remote
    |/  
    * 0dc3733 (origin/main, origin/HEAD, main, gitbutler/target) add M
    ");
    insta::assert_snapshot!(raw_json_status(&env)?, @r"
    status=exit status: 1
    stdout:

    stderr:
    Error: GitButler mode exit required: please run `but teardown` to preserve your work.
    ");

    Ok(())
}

#[test]
fn integrate_dry_run_shows_preview_without_changing_repo() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("branch-integrate-diverged")?;
    let before_log = env.git_log()?;
    let before_status = pretty_status(&env)?;

    env.but("branch update A --dry-run")
        .assert()
        .success()
        .stderr_eq(str![])
        .stdout_eq(str![[r#"
Preview

* A
● sm 74faa12 add only-on-local
● __ 28baf9a add only-on-remote
o 0dc3733

"#]]);

    insta::assert_snapshot!(before_log, @r"
    *   a952a0b (HEAD -> gitbutler/workspace) GitButler Workspace Commit
    |\  
    | * 643ade3 (A) add only-on-local
    |/  
    | * 28baf9a (origin/A) add only-on-remote
    |/  
    * 0dc3733 (origin/main, origin/HEAD, main) add M
    ");
    insta::assert_snapshot!(before_status, @r#"
    {
      "unassignedChanges": [],
      "stacks": [],
      "mergeBase": {
        "cliId": "",
        "commitId": "0dc37334a458df421bf67ea806103bf5004845dd",
        "createdAt": "2000-01-01T00:00:00+00:00",
        "message": "add M\n",
        "authorName": "author",
        "authorEmail": "author@example.com",
        "conflicted": null,
        "reviewId": null,
        "changes": null
      },
      "upstreamState": {
        "behind": 0,
        "latestCommit": {
          "cliId": "",
          "commitId": "0dc37334a458df421bf67ea806103bf5004845dd",
          "createdAt": "2000-01-01T00:00:00+00:00",
          "message": "add M\n",
          "authorName": "author",
          "authorEmail": "author@example.com",
          "conflicted": null,
          "reviewId": null,
          "changes": null
        },
        "lastFetched": null
      }
    }
    "#);
    assert_eq!(env.git_log()?, before_log, "dry-run must not rewrite refs");
    assert_eq!(
        pretty_status(&env)?,
        before_status,
        "dry-run must not change workspace status"
    );

    Ok(())
}

#[test]
fn integrate_dry_run_verbose_shows_divergence_before_preview() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("branch-integrate-diverged")?;
    let before_log = env.git_log()?;
    let before_status = pretty_status(&env)?;

    env.but("branch update A --dry-run --verbose")
        .assert()
        .success()
        .stderr_eq(str![])
        .stdout_eq(str![[r#"
Current state: A <- origin/A

● __ 643ade3 (A) add only-on-local
┊● __ 28baf9a (origin/A) add only-on-remote
├╯
o 0dc3733 add M

----------------------------

Preview

* A
● sm 74faa12 add only-on-local
● __ 28baf9a add only-on-remote
o 0dc3733

"#]]);

    assert_eq!(
        env.git_log()?,
        before_log,
        "verbose dry-run must not rewrite refs"
    );
    assert_eq!(
        pretty_status(&env)?,
        before_status,
        "verbose dry-run must not change workspace status"
    );

    Ok(())
}

#[test]
fn integrate_merge_dry_run_marks_conflicted_preview_commits() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings_slow(
        "branch-integrate-conflicting",
    )?;

    env.but("branch update A -s merge --dry-run")
        .assert()
        .success()
        .stderr_eq(str![])
        .stdout_eq(str![[r#"
Preview

* A
● __ ec97e2f Merge dbf2a866824eab2a4c485b30bcfba70af8502900 into previous commit {conflicted}
● __ 57ca948 local change in A
o 6a997fd

"#]]);

    Ok(())
}

#[test]
fn integrate_interactive_unchanged_script_applies_generated_plan() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("branch-integrate-diverged")?;
    install_editor_script(&env, "#!/usr/bin/env bash\n: \"$1\"\n")?;

    env.but("branch update A --interactive")
        .env("GIT_EDITOR", "bash editor.sh")
        .assert()
        .success()
        .stderr_eq(str![])
        .stdout_eq(str![[r#"
Updated branch A.

"#]]);

    insta::assert_snapshot!(env.git_log()?, @r"
    *   6a3496e (HEAD -> gitbutler/workspace) GitButler Workspace Commit
    |\  
    | * 74faa12 (A) add only-on-local
    | * 28baf9a (origin/A) add only-on-remote
    |/  
    * 0dc3733 (origin/main, origin/HEAD, main, gitbutler/target) add M
    ");

    Ok(())
}

#[test]
fn integrate_interactive_dry_run_keeps_repo_unchanged() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("branch-integrate-diverged")?;
    install_editor_script(&env, "#!/usr/bin/env bash\n: \"$1\"\n")?;
    let before_log = env.git_log()?;
    let before_status = pretty_status(&env)?;

    env.but("branch update A --interactive --dry-run")
        .env("GIT_EDITOR", "bash editor.sh")
        .assert()
        .success()
        .stderr_eq(str![])
        .stdout_eq(str![[r#"
Preview

* A
● sm 74faa12 add only-on-local
● __ 28baf9a add only-on-remote
o 0dc3733

"#]]);

    assert_eq!(
        env.git_log()?,
        before_log,
        "interactive dry-run must not rewrite refs"
    );
    assert_eq!(
        pretty_status(&env)?,
        before_status,
        "interactive dry-run must not change workspace status"
    );

    Ok(())
}

#[test]
fn integrate_interactive_applies_edited_merge_plan() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("branch-integrate-diverged")?;
    install_editor_script(
        &env,
        r#"#!/usr/bin/env bash
cat > "$1" <<'EOF'
pick 643ade3
merge 28baf9a
EOF
"#,
    )?;

    env.but("branch update A --interactive")
        .env("GIT_EDITOR", "bash editor.sh")
        .assert()
        .success()
        .stderr_eq(str![]);

    insta::assert_snapshot!(env.git_log()?, @r"
    *   9e0c28c (HEAD -> gitbutler/workspace) GitButler Workspace Commit
    |\  
    | *   30a8d17 (A) Merge 28baf9a2794d7722ceff84f2967b5186545b8a48 into previous commit
    | |\  
    | | * 28baf9a (origin/A) add only-on-remote
    | |/  
    |/|   
    | * 643ade3 add only-on-local
    |/  
    * 0dc3733 (origin/main, origin/HEAD, main, gitbutler/target) add M
    ");

    Ok(())
}

#[test]
fn integrate_interactive_fails_on_parse_error() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("branch-integrate-diverged")?;
    install_editor_script(
        &env,
        r#"#!/usr/bin/env bash
printf 'drop 643ade3\n' > "$1"
"#,
    )?;
    let before_log = env.git_log()?;

    env.but("branch update A --interactive")
        .env("GIT_EDITOR", "bash editor.sh")
        .assert()
        .failure()
        .stdout_eq(str![""])
        .stderr_eq(str![[r#"
Error: line 1: unknown command 'drop'

"#]]);

    assert_eq!(
        env.git_log()?,
        before_log,
        "parse failures must not rewrite refs"
    );

    Ok(())
}

#[test]
fn integrate_interactive_fails_on_out_of_scope_commit() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("branch-integrate-diverged")?;
    install_editor_script(
        &env,
        r#"#!/usr/bin/env bash
printf 'pick 0dc3733\n' > "$1"
"#,
    )?;
    let before_log = env.git_log()?;

    env.but("branch update A --interactive")
        .env("GIT_EDITOR", "bash editor.sh")
        .assert()
        .failure()
        .stdout_eq(str![""])
        .stderr_eq(str![[r#"
Error: line 1: invalid pick commit: commit '0dc3733' is not part of the editable divergence

"#]]);

    assert_eq!(
        env.git_log()?,
        before_log,
        "validation failures must not rewrite refs"
    );

    Ok(())
}

#[test]
fn integrate_errors_cleanly_without_tracking_branch() -> anyhow::Result<()> {
    let env =
        Sandbox::init_scenario_with_target_and_default_settings("branch-integrate-no-tracking")?;
    insta::assert_snapshot!(env.git_log()?, @r"
    * edd3eb7 (HEAD -> gitbutler/workspace) GitButler Workspace Commit
    * 9477ae7 (A) add A
    * 0dc3733 (origin/main, origin/HEAD, main) add M
    ");
    insta::assert_snapshot!(pretty_status(&env)?, @r#"
    {
      "unassignedChanges": [],
      "stacks": [],
      "mergeBase": {
        "cliId": "",
        "commitId": "0dc37334a458df421bf67ea806103bf5004845dd",
        "createdAt": "2000-01-01T00:00:00+00:00",
        "message": "add M\n",
        "authorName": "author",
        "authorEmail": "author@example.com",
        "conflicted": null,
        "reviewId": null,
        "changes": null
      },
      "upstreamState": {
        "behind": 0,
        "latestCommit": {
          "cliId": "",
          "commitId": "0dc37334a458df421bf67ea806103bf5004845dd",
          "createdAt": "2000-01-01T00:00:00+00:00",
          "message": "add M\n",
          "authorName": "author",
          "authorEmail": "author@example.com",
          "conflicted": null,
          "reviewId": null,
          "changes": null
        },
        "lastFetched": null
      }
    }
    "#);

    env.but("branch update A")
        .assert()
        .failure()
        .stdout_eq(str![""])
        .stderr_eq(str![[r#"
Error: Branch 'refs/heads/A' has no tracking branch

"#]]);

    Ok(())
}

/// Sprint-04 red-hat G5: `but branch integrate` (the CLI surface of
/// `apply_branch_integration`) must be gated by the commit gate. A read-only
/// principal on a governed workspace must be denied `perm.denied` naming
/// `contents:write`; the workspace ref must not advance.
#[test]
#[serial_test::serial]
fn integrate_readonly_denied() -> anyhow::Result<()> {
    let env = governed_integrate_env()?;
    let repo = env.open_repo()?;
    let workspace_before = ref_id(&repo, "refs/heads/gitbutler/workspace")?;

    let output = env
        .but("branch update A")
        .env("BUT_AGENT_HANDLE", "ro")
        .output()?;
    assert_eq!(
        output.status.code(),
        Some(1),
        "read-only integrate must exit 1; stdout: {}; stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(r#""code":"perm.denied""#) || stderr.contains("contents:write"),
        "read-only integrate must surface perm.denied naming contents:write; stderr: {stderr}"
    );
    assert_eq!(
        ref_id(&repo, "refs/heads/gitbutler/workspace")?,
        workspace_before,
        "denied integrate must leave the workspace ref unchanged"
    );

    println!("read-only integrate denied by commit gate; workspace ref unchanged");
    Ok(())
}

fn governed_integrate_env() -> anyhow::Result<Sandbox> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("branch-integrate-diverged")?;
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
    Ok(env)
}

fn ref_id(repo: &gix::Repository, ref_name: &str) -> anyhow::Result<gix::ObjectId> {
    Ok(repo.find_reference(ref_name)?.peel_to_id()?.detach())
}
