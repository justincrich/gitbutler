use but_api::legacy::config_mutate::{
    AdminWriteGateError, classify_error, enforce_administration_write_gate,
};

const MAIN_REF: &str = "refs/heads/main";
const PERMISSIONS_PATH: &str = ".gitbutler/permissions.toml";

#[test]
#[serial_test::serial]
fn admin_write_guard_denies_non_admin_allows_admin() -> anyhow::Result<()> {
    let (repo, _tmp) = admin_write_repo();

    let dev_error =
        temp_env::with_var("BUT_AGENT_HANDLE", Some("dev"), || -> anyhow::Result<_> {
            classified_error(enforce_administration_write_gate(&repo, MAIN_REF))
        })?;
    assert_eq!(
        dev_error.code, "perm.denied",
        "principal without administration:write must be denied"
    );
    assert!(
        dev_error.message.contains("administration:write"),
        "denial message must name the missing administration:write authority"
    );

    temp_env::with_var(
        "BUT_AGENT_HANDLE",
        Some("admin"),
        || -> anyhow::Result<()> { enforce_administration_write_gate(&repo, MAIN_REF) },
    )?;

    let committed_before = committed_blob_text(&repo, PERMISSIONS_PATH)?;
    write_worktree_permissions(
        &repo,
        r#"
[[principal]]
id = "dev"
permissions = ["contents:write", "administration:write"]

[[principal]]
id = "admin"
permissions = ["administration:write", "merge"]
"#,
    )?;

    let self_grant_error =
        temp_env::with_var("BUT_AGENT_HANDLE", Some("dev"), || -> anyhow::Result<_> {
            classified_error(enforce_administration_write_gate(&repo, MAIN_REF))
        })?;
    assert_eq!(
        self_grant_error.code, "perm.denied",
        "uncommitted working-tree permissions.toml must not widen target-ref authority"
    );
    assert!(
        self_grant_error.message.contains("administration:write"),
        "working-tree self-grant denial must still name administration:write"
    );
    assert_eq!(
        committed_blob_text(&repo, PERMISSIONS_PATH)?,
        committed_before,
        "target-ref permissions.toml must remain committed with dev lacking administration:write"
    );

    println!("dev denied with `error.code == \"perm.denied\"`");
    println!("admin permitted with `Ok(())`");
    println!("working-tree self-grant still denied with `error.code == \"perm.denied\"`");
    Ok(())
}

#[test]
#[serial_test::serial]
fn admin_write_guard_malformed_config_invalid() -> anyhow::Result<()> {
    let (repo, _tmp) = admin_write_malformed();

    let error = temp_env::with_var(
        "BUT_AGENT_HANDLE",
        Some("admin"),
        || -> anyhow::Result<_> {
            classified_error(enforce_administration_write_gate(&repo, MAIN_REF))
        },
    )?;

    assert_eq!(
        error.code, "config.invalid",
        "malformed committed governance config must fail closed as config.invalid"
    );
    assert_ne!(
        error.code, "perm.denied",
        "malformed config must not be blurred into a permission denial"
    );

    println!("malformed target-ref permissions.toml returned `config.invalid`");
    Ok(())
}

fn admin_write_repo() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "dev"
permissions = ["contents:write"]

[[principal]]
id = "admin"
permissions = ["administration:write", "merge"]
EOF

cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF

git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "admin write governance config"
"#,
        &repo,
    );
    (repo, tmp)
}

fn admin_write_malformed() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]
id = "admin"
permissions = nope
EOF

cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF

git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "malformed admin write governance config"
"#,
        &repo,
    );
    (repo, tmp)
}

fn classified_error(result: anyhow::Result<()>) -> anyhow::Result<AdminWriteGateError> {
    match result {
        Ok(()) => anyhow::bail!("administration write gate should reject this scenario"),
        Err(error) => classify_error(&error)
            .ok_or_else(|| anyhow::anyhow!("administration write gate error should classify")),
    }
}

fn write_worktree_permissions(repo: &gix::Repository, contents: &str) -> anyhow::Result<()> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow::anyhow!("test repository must be non-bare"))?;
    std::fs::write(workdir.join(PERMISSIONS_PATH), contents)?;
    Ok(())
}

fn committed_blob_text(repo: &gix::Repository, path: &str) -> anyhow::Result<String> {
    let mut reference = repo.find_reference(MAIN_REF)?;
    let commit = reference.peel_to_commit()?;
    let tree = commit.tree()?;
    let entry = tree
        .lookup_entry_by_path(std::path::Path::new(path))?
        .ok_or_else(|| anyhow::anyhow!("expected {path} in committed target ref"))?;
    let blob = repo.find_object(entry.id())?.try_into_blob()?;
    Ok(String::from_utf8(blob.data.to_vec())?)
}

/// Sprint-02 red-hat G-4: a ghost caller (handle not in committed config) hitting
/// a malformed governance config must surface `config.invalid` -- NOT
/// `perm.denied`. Proves the admin-write guard runs config-load BEFORE
/// authorize (deterministic fail-closed ordering), mirroring AUTHZ-004 AC-2's
/// coverage of the merge-gate path.
#[test]
#[serial_test::serial]
fn admin_write_guard_malformed_config_invalid_for_ghost_caller() -> anyhow::Result<()> {
    let (repo, _tmp) = admin_write_malformed();

    // BUT_AGENT_HANDLE names a principal that does NOT exist in the
    // (malformed) committed config. If authorize ran before config-load,
    // this would surface `perm.denied` for the unknown principal.
    let error = temp_env::with_var(
        "BUT_AGENT_HANDLE",
        Some("ghost-not-in-config"),
        || -> anyhow::Result<_> {
            classified_error(enforce_administration_write_gate(&repo, MAIN_REF))
        },
    )?;

    assert_eq!(
        error.code, "config.invalid",
        "malformed committed config must surface config.invalid even for an unknown caller -- \
         config-load must run BEFORE authorize (deterministic fail-closed ordering)"
    );
    assert_ne!(
        error.code, "perm.denied",
        "an unknown principal must NOT be blamed when the config itself is malformed"
    );

    println!("ghost caller + malformed config returned `config.invalid` (config-load-first)");
    Ok(())
}
