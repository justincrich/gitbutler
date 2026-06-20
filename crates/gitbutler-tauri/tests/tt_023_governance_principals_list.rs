//! TT-023 governance principals list renderer contract proofs.
//!
//! These tests exercise Tauri's real mock runtime command bus with
//! `tauri::test::get_ipc_response`; the command reads real committed target-ref
//! config and real working-tree config bytes.

use std::fs;

use anyhow::Context as _;
use serde_json::{Value, json};
use tauri::{WebviewWindowBuilder, ipc::InvokeBody, webview::InvokeRequest};

const MAIN_REF: &str = "refs/heads/main";
const TARGET_REF: &str = "refs/remotes/origin/main";
const PERMISSIONS_PATH: &str = ".gitbutler/permissions.toml";
const GATES_PATH: &str = ".gitbutler/gates.toml";

#[test]
#[serial_test::serial]
fn governance_principals_list_reports_committed_display_with_direct_pending_only()
-> anyhow::Result<()> {
    let (repo, _tmp) = governance_repo(
        r#"
[[principal]]
id = "alice"
permissions = ["contents:read"]

[[principal]]
id = "codex-agent"
permissions = ["reviews:write"]

[[group]]
name = "eng"
permissions = ["contents:write"]
members = ["codex-agent", "bob"]
"#,
    );
    write_worktree_permissions(
        &repo,
        r#"
[[principal]]
id = "alice"
permissions = ["contents:read", "reviews:write"]

[[principal]]
id = "codex-agent"
permissions = ["reviews:write"]

[[principal]]
id = "worktree-only"
permissions = ["contents:read"]

[[group]]
name = "eng"
permissions = ["contents:write"]
members = ["codex-agent", "bob"]

[[group]]
name = "qa"
permissions = ["reviews:write"]
members = ["inherited-only"]
"#,
    )?;

    let response = invoke_principals_list(&repo)?;
    let principals = response
        .get("principals")
        .and_then(Value::as_array)
        .context("response must contain principals array")?;
    assert_eq!(
        principals.len(),
        5,
        "must include committed direct principals, committed group members, working direct principals, and working group members: {response:?}"
    );

    let alice = principal(&response, "alice")?;
    assert_eq!(
        alice.get("ownGrants"),
        Some(&json!(["contents:read"])),
        "ownGrants must stay committed and exclude uncommitted direct reviews:write"
    );
    assert_eq!(
        alice.get("groupMemberships"),
        Some(&json!([])),
        "alice has no committed group membership"
    );
    assert_eq!(
        alice.get("pending").and_then(Value::as_bool),
        Some(true),
        "direct working-tree grant must mark alice pending"
    );

    let codex_agent = principal(&response, "codex-agent")?;
    assert_eq!(
        codex_agent.get("ownGrants"),
        Some(&json!(["reviews:write"])),
        "codex-agent direct committed grant must be present"
    );
    assert_eq!(
        codex_agent.get("groupMemberships"),
        Some(&json!(["eng"])),
        "codex-agent committed group membership must include eng"
    );
    assert_eq!(
        codex_agent.get("inheritedGrants"),
        Some(&json!([{ "authority": "contents:write", "sourceLabel": "group: eng" }])),
        "codex-agent must inherit contents:write from eng with a renderer source label"
    );
    assert_eq!(
        codex_agent.get("pending").and_then(Value::as_bool),
        Some(false),
        "unchanged direct grants must not be pending"
    );

    let bob = principal(&response, "bob")?;
    assert_eq!(
        bob.get("ownGrants"),
        Some(&json!([])),
        "group-only committed member must have no direct own grants"
    );
    assert_eq!(
        bob.get("groupMemberships"),
        Some(&json!(["eng"])),
        "group-only committed member must expose committed membership"
    );
    assert_eq!(
        bob.get("pending").and_then(Value::as_bool),
        Some(false),
        "inherited-only principal must not be pending"
    );

    let worktree_only = principal(&response, "worktree-only")?;
    assert_eq!(
        worktree_only.get("ownGrants"),
        Some(&json!([])),
        "working-tree-only principal must not leak uncommitted grants into committed display"
    );
    assert_eq!(
        worktree_only.get("groupMemberships"),
        Some(&json!([])),
        "working-tree-only principal must not show uncommitted groups as committed"
    );
    assert_eq!(
        worktree_only.get("pending").and_then(Value::as_bool),
        Some(true),
        "working-tree-only direct principal must be pending"
    );

    let inherited_only = principal(&response, "inherited-only")?;
    assert_eq!(
        inherited_only.get("ownGrants"),
        Some(&json!([])),
        "working-tree group-only principal has no committed direct grants"
    );
    assert_eq!(
        inherited_only.get("groupMemberships"),
        Some(&json!([])),
        "working-tree group-only principal has no committed memberships"
    );
    assert_eq!(
        inherited_only.get("pending").and_then(Value::as_bool),
        Some(false),
        "working-tree inherited-only principal must not be pending"
    );
    Ok(())
}

fn invoke_principals_list(repo: &gix::Repository) -> anyhow::Result<Value> {
    let project_id = project_id_for(repo)?;
    let app = governance_app()?;
    let webview = governance_webview(&app)?;
    invoke(
        &webview,
        "governance_principals_list",
        json!({ "projectId": project_id, "targetRef": TARGET_REF }),
    )?
    .map_err(|error| anyhow::anyhow!("unexpected IPC error: {error:?}"))
}

fn governance_app() -> anyhow::Result<tauri::App<tauri::test::MockRuntime>> {
    macro_rules! tauri_handler_from_governance_rows {
        ($($governance_command:path),* $(,)?) => {
            tauri::generate_handler![$($governance_command),*]
        };
    }

    tauri::test::mock_builder()
        .manage(gitbutler_tauri::governance::DesktopSessionState::new(
            gitbutler_tauri::governance::TestDesktopSession {
                user_id: 42,
                login: Some("fleet-owner".to_owned()),
            },
        ))
        .invoke_handler(gitbutler_tauri::gitbutler_governance_command_rows!(
            tauri_handler_from_governance_rows
        ))
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .map_err(Into::into)
}

fn governance_webview(
    app: &tauri::App<tauri::test::MockRuntime>,
) -> anyhow::Result<tauri::WebviewWindow<tauri::test::MockRuntime>> {
    WebviewWindowBuilder::new(app, "main", Default::default())
        .build()
        .map_err(Into::into)
}

fn invoke(
    webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
    command: &str,
    payload: Value,
) -> anyhow::Result<Result<Value, Value>> {
    let request = InvokeRequest {
        cmd: command.into(),
        callback: tauri::ipc::CallbackFn(0),
        error: tauri::ipc::CallbackFn(1),
        url: "tauri://localhost".parse()?,
        body: InvokeBody::from(payload),
        headers: Default::default(),
        invoke_key: tauri::test::INVOKE_KEY.to_owned(),
    };

    match tauri::test::get_ipc_response(webview, request) {
        Ok(body) => Ok(Ok(body.deserialize::<Value>()?)),
        Err(error) => Ok(Err(error)),
    }
}

fn principal<'a>(response: &'a Value, id: &str) -> anyhow::Result<&'a Value> {
    response
        .get("principals")
        .and_then(Value::as_array)
        .and_then(|principals| {
            principals
                .iter()
                .find(|principal| principal.get("principalId").and_then(Value::as_str) == Some(id))
        })
        .with_context(|| format!("response must include principal {id}: {response:?}"))
}

fn governance_repo(permissions: &str) -> (gix::Repository, tempfile::TempDir) {
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
mkdir -p .gitbutler
"#,
        tmp.path(),
    );
    let repo = gix::open(tmp.path())
        .unwrap_or_else(|error| panic!("opening {} failed: {error}", tmp.path().display()));
    write_worktree_permissions(&repo, permissions)
        .unwrap_or_else(|error| panic!("writing seed permissions failed: {error}"));
    write_worktree_gates(
        &repo,
        r#"
[[branch]]
name = "main"
protected = true
"#,
    )
    .unwrap_or_else(|error| panic!("writing seed gates failed: {error}"));
    but_testsupport::invoke_bash(
        &format!(
            r#"
git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "governance ipc config"
git update-ref {TARGET_REF} {MAIN_REF}
"#
        ),
        &repo,
    );
    (repo, tmp)
}

fn write_worktree_permissions(repo: &gix::Repository, contents: &str) -> anyhow::Result<()> {
    fs::write(worktree_path(repo, PERMISSIONS_PATH)?, contents).map_err(Into::into)
}

fn write_worktree_gates(repo: &gix::Repository, contents: &str) -> anyhow::Result<()> {
    fs::write(worktree_path(repo, GATES_PATH)?, contents).map_err(Into::into)
}

fn worktree_path(repo: &gix::Repository, path: &str) -> anyhow::Result<std::path::PathBuf> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow::anyhow!("test repository must be non-bare"))?;
    Ok(workdir.join(path))
}

fn project_id_for(
    repo: &gix::Repository,
) -> anyhow::Result<but_ctx::ProjectHandleOrLegacyProjectId> {
    let handle = but_ctx::ProjectHandle::from_path(repo.git_dir())?;
    Ok(but_ctx::ProjectHandleOrLegacyProjectId::ProjectHandle(
        handle,
    ))
}
