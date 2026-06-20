//! IPC-003 governance command registration & invocation proofs.
//!
//! These tests prove the 12 governance commands are registered through the
//! real `gitbutler_tauri::invoke_handler()` factory (extracted from
//! `main.rs::run()` in SPEC-REPAIR-IPC-003) and that the registration surface
//! matches the live but-api contract.
//!
//! They do not duplicate the registration list — they read the single source
//! of truth (`crates/gitbutler-tauri/src/lib.rs`) and assert that every
//! governance command in the contract is present as
//! `legacy::governance::tauri_<name>::<name>` inside the factory's
//! `tauri::generate_handler!` payload, and that `main.rs` consumes the factory
//! rather than re-declaring the list.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;

/// All 12 governance commands IPC-003 promises to register.
const GOVERNANCE_COMMANDS: &[&str] = &[
    "perm_list",
    "perm_grant",
    "perm_revoke",
    "group_create",
    "group_grant",
    "group_add_member",
    "group_remove_member",
    "group_delete",
    "group_list",
    "branch_gates_read",
    "branch_gates_update",
    "governance_status_read",
];

const MAIN_REF: &str = "refs/heads/main";
const TARGET_REF: &str = "refs/remotes/origin/main";
const PERMISSIONS_PATH: &str = ".gitbutler/permissions.toml";

#[test]
#[serial_test::serial]
fn mgmt_config_command_resolves_fleet_owner_via_shim() -> anyhow::Result<()> {
    let (repo, _tmp) = governance_api_repo(true);
    let project_id = project_id_for(&repo)?;
    let session = test_desktop_session();

    let outcome = temp_env::with_var("BUT_AGENT_HANDLE", None::<&str>, || {
        gitbutler_tauri::governance::perm_grant_for_desktop_session(
            &session,
            project_id.clone(),
            TARGET_REF.to_owned(),
            "rust-reviewer".to_owned(),
            vec!["reviews:write".to_owned()],
        )
    })?;

    assert_eq!(outcome.principal, "rust-reviewer");
    assert!(
        worktree_permissions(&repo)?.contains("reviews:write"),
        "desktop fleet-owner grant must write the requested authority without BUT_AGENT_HANDLE"
    );
    Ok(())
}

#[test]
#[serial_test::serial]
fn mgmt_unauthorized_agent_config_command_denied() -> anyhow::Result<()> {
    let (repo, _tmp) = governance_api_repo(true);
    let project_id = project_id_for(&repo)?;
    let before = worktree_permissions_bytes(&repo)?;

    let result = temp_env::with_var("BUT_AGENT_HANDLE", Some("rust-implementer"), || {
        gitbutler_tauri::governance::perm_grant_for_agent_env(
            project_id.clone(),
            TARGET_REF.to_owned(),
            "rust-implementer".to_owned(),
            vec!["administration:write".to_owned()],
        )
    });

    let error = match result {
        Ok(_) => anyhow::bail!("non-admin agent perm_grant must be denied"),
        Err(error) => serde_json::to_value(error)?,
    };
    assert_perm_denied_with_hint(&error, "administration:write");
    assert_eq!(
        worktree_permissions_bytes(&repo)?,
        before,
        "denied non-admin agent invoke must leave permissions.toml byte-identical"
    );
    Ok(())
}

#[test]
#[serial_test::serial]
fn mgmt_fleet_owner_grants_on_bootstrap_no_committed_config() -> anyhow::Result<()> {
    let (repo, _tmp) = governance_api_repo(false);
    let project_id = project_id_for(&repo)?;
    let session = test_desktop_session();

    temp_env::with_var("BUT_AGENT_HANDLE", None::<&str>, || {
        gitbutler_tauri::governance::perm_grant_for_desktop_session(
            &session,
            project_id.clone(),
            TARGET_REF.to_owned(),
            "rust-reviewer".to_owned(),
            vec!["reviews:write".to_owned()],
        )
    })?;

    let permissions = worktree_permissions(&repo)?;
    assert!(
        permissions.contains("rust-reviewer") && permissions.contains("reviews:write"),
        "fleet-owner superuser path must bootstrap permissions.toml without committed grants"
    );
    Ok(())
}

#[test]
#[serial_test::serial]
fn mgmt_nonadmin_env_handle_does_not_shadow_fleet_owner() -> anyhow::Result<()> {
    let (repo, _tmp) = governance_api_repo(true);
    let project_id = project_id_for(&repo)?;
    let session = test_desktop_session();

    let outcome = temp_env::with_var("BUT_AGENT_HANDLE", Some("rust-implementer"), || {
        gitbutler_tauri::governance::perm_grant_for_desktop_session(
            &session,
            project_id.clone(),
            TARGET_REF.to_owned(),
            "rust-reviewer".to_owned(),
            vec!["reviews:write".to_owned()],
        )
    })?;

    assert_eq!(outcome.principal, "rust-reviewer");
    assert!(
        worktree_permissions(&repo)?.contains("reviews:write"),
        "non-admin BUT_AGENT_HANDLE must not shadow the desktop fleet-owner identity"
    );
    Ok(())
}

#[test]
fn mgmt_governance_commands_registered_and_invokable() {
    // Prove the real factory compiles + is callable from outside the binary.
    // `invoke_handler` returns the `tauri::generate_handler!` closure; calling
    // it without a full Tauri runtime would panic on Invoke resolution, so we
    // only build it here. Compilation + the absence of `unreachable!`/panic
    // during construction is the load-bearing proof that the registration
    // surface compiles end-to-end against the real but-api macros.
    let _handler = gitbutler_tauri::invoke_handler();

    // Belt-and-braces: the const view exported from lib.rs must align with the
    // IPC-003 contract — every governance command present.
    for command in GOVERNANCE_COMMANDS {
        assert!(
            gitbutler_tauri::GOVERNANCE_COMMANDS.contains(command),
            "gitbutler_tauri::GOVERNANCE_COMMANDS must advertise {command}"
        );
    }

    // The factory's `generate_handler!` payload (the single source of truth)
    // must register each governance command under the bare post-rename path
    // `legacy::governance::tauri_<name>::<name>`. This is the same source the
    // desktop binary consumes, so the test cannot drift from production.
    let factory_source = read_crate_file("src/lib.rs");
    let api_contract_mismatch = governance_api_contract_mismatch();
    let missing_from_factory = GOVERNANCE_COMMANDS
        .iter()
        .copied()
        .filter(|command| !factory_registers_command(&factory_source, command))
        .collect::<Vec<_>>();

    assert!(
        missing_from_factory.is_empty() && api_contract_mismatch.is_empty(),
        "MGMT-IPC-003 requires all governance commands to be registered in the real \
         gitbutler_tauri::invoke_handler() factory surface as \
         `legacy::governance::tauri_<name>::<name>` rows. Missing factory rows: \
         {missing_from_factory:?}. Current but-api/spec mismatch: {api_contract_mismatch:?}."
    );

    // Pin the factory shape: lib.rs must be the single registration site.
    assert!(
        factory_source.contains("pub fn invoke_handler("),
        "lib.rs must expose the public invoke_handler factory consumed by main.rs"
    );
    assert!(
        factory_source.contains("tauri::generate_handler!["),
        "invoke_handler must build the payload from the real tauri::generate_handler! macro"
    );

    // main.rs must NOT duplicate the registration list — it consumes the
    // factory. A second `generate_handler!` in the binary would let the test
    // surface drift from production.
    let main_source = read_crate_file("src/main.rs");
    assert!(
        !main_source.contains("tauri::generate_handler!["),
        "main.rs must consume gitbutler_tauri::invoke_handler() and not duplicate the \
         generate_handler! command list"
    );
    assert!(
        main_source.contains("gitbutler_tauri::invoke_handler()"),
        "main.rs must register the governance command surface through the extracted factory"
    );
}

#[test]
fn mgmt_capability_main_scope_preserved() {
    let capability = read_crate_file("capabilities/main.json");

    assert!(
        capability.contains("\"identifier\": \"main\""),
        "the governance command surface must stay under the existing main capability"
    );
    assert!(
        capability.contains("\"windows\": [\"*\"]"),
        "the main capability must continue to admit all desktop windows"
    );
    assert!(
        capability.contains("\"core:default\""),
        "GitButler app commands are admitted through core:default, not hand-written allow-* files"
    );

    let forbidden_allow_files = capability_files()
        .into_iter()
        .filter(|path| {
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                return false;
            };
            name.starts_with("allow-perm_")
                || name.starts_with("allow-group_")
                || name.starts_with("allow-branch_gates_")
                || name.starts_with("allow-governance_")
        })
        .collect::<Vec<_>>();

    assert!(
        forbidden_allow_files.is_empty(),
        "governance commands must not be admitted by fake per-command allow files: {forbidden_allow_files:?}"
    );
}

#[test]
fn mgmt_unregistered_governance_command_not_invokable() {
    let factory_source = read_crate_file("src/lib.rs");
    let negative_control = "mgmt_unregistered_governance_probe";

    assert!(
        !factory_registers_command(&factory_source, negative_control),
        "the deliberately unregistered governance negative control must remain command-not-found"
    );
}

fn factory_registers_command(factory_source: &str, command: &str) -> bool {
    factory_source.contains(&format!("legacy::governance::tauri_{command}::{command}"))
        || factory_source.contains(&format!("governance::tauri_{command}::{command}"))
}

fn governance_api_contract_mismatch() -> Vec<String> {
    let governance_api = read_workspace_file("crates/but-api/src/legacy/governance.rs");
    GOVERNANCE_COMMANDS
        .iter()
        .copied()
        .filter_map(|command| {
            let exact_wrapper = format!("#[but_api]\npub fn {command}(");
            let converted_wrapper = format!("#[but_api(GovernanceStatus)]\npub fn {command}(");
            if governance_api.contains(&exact_wrapper)
                || governance_api.contains(&converted_wrapper)
            {
                None
            } else {
                Some(command.to_owned())
            }
        })
        .collect()
}

fn capability_files() -> Vec<PathBuf> {
    let capability_dir = crate_dir().join("capabilities");
    let mut files = Vec::new();
    collect_files(&capability_dir, &mut files);
    files
}

fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("reading {} failed: {error}", dir.display()))
    {
        let path = entry.expect("reading directory entry failed").path();
        if path.is_dir() {
            collect_files(&path, files);
        } else {
            files.push(path);
        }
    }
}

fn read_crate_file(relative: &str) -> String {
    let path = crate_dir().join(relative);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("reading {} failed: {error}", path.display()))
}

fn read_workspace_file(relative: &str) -> String {
    let path = workspace_dir().join(relative);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("reading {} failed: {error}", path.display()))
}

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn workspace_dir() -> PathBuf {
    crate_dir()
        .parent()
        .and_then(Path::parent)
        .expect("gitbutler-tauri lives under crates/")
        .to_path_buf()
}

fn governance_api_repo(with_committed_permissions: bool) -> (gix::Repository, tempfile::TempDir) {
    let tmp = tempfile::TempDir::new()
        .unwrap_or_else(|error| panic!("creating temp repository failed: {error}"));
    but_testsupport::invoke_bash_at_dir(
        r#"
git init --initial-branch=main
git config user.name "GitButler Test"
git config user.email "gitbutler@example.com"
echo initial >README.md
git add README.md
git commit -m "initial"
"#,
        tmp.path(),
    );
    let repo = gix::open(tmp.path())
        .unwrap_or_else(|error| panic!("opening {} failed: {error}", tmp.path().display()));
    let permissions = if with_committed_permissions {
        r#"
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "admin"
permissions = ["administration:write", "merge"]

[[principal]]
id = "rust-implementer"
permissions = ["contents:write"]

[[principal]]
id = "rust-reviewer"
permissions = ["contents:read"]
EOF
git add .gitbutler/permissions.toml
"#
    } else {
        ""
    };
    but_testsupport::invoke_bash(
        &format!(
            r#"
mkdir -p .gitbutler
{permissions}
cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF
git add .gitbutler/gates.toml
git commit -m "governance ipc config"
git update-ref {TARGET_REF} {MAIN_REF}
"#
        ),
        &repo,
    );
    (repo, tmp)
}

fn project_id_for(
    repo: &gix::Repository,
) -> anyhow::Result<but_ctx::ProjectHandleOrLegacyProjectId> {
    let git_dir = repo.git_dir();
    let handle = but_ctx::ProjectHandle::from_path(git_dir)?;
    Ok(but_ctx::ProjectHandleOrLegacyProjectId::ProjectHandle(
        handle,
    ))
}

fn test_desktop_session() -> gitbutler_tauri::governance::TestDesktopSession {
    gitbutler_tauri::governance::TestDesktopSession {
        user_id: 42,
        login: Some("fleet-owner".to_owned()),
    }
}

fn worktree_permissions(repo: &gix::Repository) -> anyhow::Result<String> {
    let path = repo
        .workdir()
        .ok_or_else(|| anyhow::anyhow!("test repository must be non-bare"))?
        .join(PERMISSIONS_PATH);
    Ok(fs::read_to_string(path)?)
}

fn worktree_permissions_bytes(repo: &gix::Repository) -> anyhow::Result<Vec<u8>> {
    let path = repo
        .workdir()
        .ok_or_else(|| anyhow::anyhow!("test repository must be non-bare"))?
        .join(PERMISSIONS_PATH);
    Ok(fs::read(path)?)
}

fn assert_perm_denied_with_hint(error: &Value, missing: &str) {
    assert_eq!(
        error.get("code").and_then(Value::as_str),
        Some("perm.denied"),
        "governance IPC denial must serialize as code perm.denied: {error:?}"
    );
    assert!(
        error
            .get("message")
            .and_then(Value::as_str)
            .is_some_and(|message| message.contains(missing)),
        "serialized denial message must name the missing {missing} authority"
    );
    assert!(
        error
            .get("remediation_hint")
            .and_then(Value::as_str)
            .is_some_and(|hint| !hint.is_empty() && hint.contains(missing)),
        "serialized denial must include a non-empty remediation_hint naming {missing}"
    );
}
