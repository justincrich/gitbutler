//! IPC-003 governance command registration and invocation proofs.
//!
//! These tests exercise Tauri's real mock runtime command bus with
//! `tauri::test::get_ipc_response`. They intentionally avoid source-only
//! registration checks for the behavioral ACs.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::{Value, json};
use tauri::{WebviewWindowBuilder, ipc::InvokeBody, webview::InvokeRequest};

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
    let app = governance_app(test_desktop_session())?;
    let webview = governance_webview(&app)?;

    let response = temp_env::with_var("BUT_AGENT_HANDLE", None::<&str>, || {
        invoke_ok(
            &webview,
            "perm_grant",
            grant_payload(&project_id, "rust-reviewer", ["reviews:write"]),
        )
    })?;

    assert_eq!(
        response.get("principal").and_then(Value::as_str),
        Some("rust-reviewer")
    );
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
    let app = governance_app(test_desktop_session())?;
    let webview = governance_webview(&app)?;

    let error = temp_env::with_var("BUT_AGENT_HANDLE", Some("rust-implementer"), || {
        invoke_err(
            &webview,
            "agent_perm_grant",
            grant_payload(&project_id, "rust-implementer", ["administration:write"]),
        )
    })?;

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
    let app = governance_app(test_desktop_session())?;
    let webview = governance_webview(&app)?;

    temp_env::with_var("BUT_AGENT_HANDLE", None::<&str>, || {
        invoke_ok(
            &webview,
            "perm_grant",
            grant_payload(&project_id, "rust-reviewer", ["reviews:write"]),
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
    let app = governance_app(test_desktop_session())?;
    let webview = governance_webview(&app)?;

    let response = temp_env::with_var("BUT_AGENT_HANDLE", Some("rust-implementer"), || {
        invoke_ok(
            &webview,
            "perm_grant",
            grant_payload(&project_id, "rust-reviewer", ["reviews:write"]),
        )
    })?;

    assert_eq!(
        response.get("principal").and_then(Value::as_str),
        Some("rust-reviewer")
    );
    assert!(
        worktree_permissions(&repo)?.contains("reviews:write"),
        "non-admin BUT_AGENT_HANDLE must not shadow the desktop fleet-owner identity"
    );
    Ok(())
}

#[test]
#[serial_test::serial]
fn mgmt_governance_commands_registered_and_invokable() -> anyhow::Result<()> {
    let (repo, _tmp) = governance_api_repo(true);
    let project_id = project_id_for(&repo)?;
    let app = governance_app(test_desktop_session())?;
    let webview = governance_webview(&app)?;

    let reached = temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        let mut reached = Vec::new();
        for case in governance_invocation_cases(&project_id) {
            let result = invoke(&webview, case.command, case.payload.clone())?;
            assert!(
                result.is_ok() || structured_governance_error(result.as_ref().err()),
                "{} must reach its registered governance command and return a real result/error, got {result:?}",
                case.command
            );
            reached.push(case.command);
        }
        anyhow::Ok(reached)
    })?;

    assert_eq!(
        reached, GOVERNANCE_COMMANDS,
        "the IPC harness must invoke exactly the 12 governance commands in the contract"
    );
    assert!(
        worktree_permissions(&repo)?.contains("reviews:write"),
        "perm_grant IPC call must reach the real mutation path"
    );
    Ok(())
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
#[serial_test::serial]
fn mgmt_unregistered_governance_command_not_invokable() -> anyhow::Result<()> {
    let app = governance_app(test_desktop_session())?;
    let webview = governance_webview(&app)?;
    let error = invoke_err(&webview, "mgmt_unregistered_governance_probe", json!({}))?;

    assert!(
        error.to_string().contains("not found"),
        "the real Tauri bus must reject an unregistered governance command, got {error:?}"
    );
    Ok(())
}

struct InvocationCase {
    command: &'static str,
    payload: Value,
}

fn governance_invocation_cases(
    project_id: &but_ctx::ProjectHandleOrLegacyProjectId,
) -> Vec<InvocationCase> {
    vec![
        InvocationCase {
            command: "perm_list",
            payload: json!({ "projectId": project_id, "principal": "admin" }),
        },
        InvocationCase {
            command: "perm_grant",
            payload: grant_payload(project_id, "rust-reviewer", ["reviews:write"]),
        },
        InvocationCase {
            command: "perm_revoke",
            payload: grant_payload(project_id, "rust-reviewer", ["contents:read"]),
        },
        InvocationCase {
            command: "group_create",
            payload: json!({
                "projectId": project_id,
                "targetRef": TARGET_REF,
                "group": "qa",
                "authorities": ["reviews:write"]
            }),
        },
        InvocationCase {
            command: "group_grant",
            payload: json!({
                "projectId": project_id,
                "targetRef": TARGET_REF,
                "group": "qa",
                "authorities": ["contents:write"]
            }),
        },
        InvocationCase {
            command: "group_add_member",
            payload: json!({
                "projectId": project_id,
                "targetRef": TARGET_REF,
                "group": "qa",
                "member": "rust-reviewer"
            }),
        },
        InvocationCase {
            command: "group_remove_member",
            payload: json!({
                "projectId": project_id,
                "targetRef": TARGET_REF,
                "group": "qa",
                "member": "rust-reviewer"
            }),
        },
        InvocationCase {
            command: "group_delete",
            payload: json!({
                "projectId": project_id,
                "targetRef": TARGET_REF,
                "group": "qa"
            }),
        },
        InvocationCase {
            command: "group_list",
            payload: json!({ "projectId": project_id }),
        },
        InvocationCase {
            command: "branch_gates_read",
            payload: json!({ "projectId": project_id, "targetRef": TARGET_REF }),
        },
        InvocationCase {
            command: "branch_gates_update",
            payload: json!({
                "projectId": project_id,
                "targetRef": TARGET_REF,
                "branch": "release",
                "protection": { "protected": true }
            }),
        },
        InvocationCase {
            command: "governance_status_read",
            payload: json!({ "projectId": project_id }),
        },
    ]
}

fn grant_payload(
    project_id: &but_ctx::ProjectHandleOrLegacyProjectId,
    principal: &str,
    authorities: impl IntoIterator<Item = &'static str>,
) -> Value {
    json!({
        "projectId": project_id,
        "targetRef": TARGET_REF,
        "principal": principal,
        "authorities": authorities.into_iter().collect::<Vec<_>>()
    })
}

fn governance_app(
    session: gitbutler_tauri::governance::TestDesktopSession,
) -> anyhow::Result<tauri::App<tauri::test::MockRuntime>> {
    macro_rules! tauri_handler_from_governance_rows {
        ($($governance_command:path),* $(,)?) => {
            tauri::generate_handler![$($governance_command),*]
        };
    }

    tauri::test::mock_builder()
        .manage(gitbutler_tauri::governance::DesktopSessionState::new(
            session,
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

fn invoke_ok(
    webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
    command: &str,
    payload: Value,
) -> anyhow::Result<Value> {
    invoke(webview, command, payload)?
        .map_err(|error| anyhow::anyhow!("unexpected IPC error: {error:?}"))
}

fn invoke_err(
    webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
    command: &str,
    payload: Value,
) -> anyhow::Result<Value> {
    match invoke(webview, command, payload)? {
        Ok(value) => anyhow::bail!("expected IPC error from {command}, got {value:?}"),
        Err(error) => Ok(error),
    }
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

fn structured_governance_error(error: Option<&Value>) -> bool {
    error.is_some_and(|error| {
        error
            .get("code")
            .and_then(Value::as_str)
            .is_some_and(|code| matches!(code, "perm.denied" | "config.invalid"))
            || !error.to_string().contains("not found")
    })
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
        let path = entry
            .unwrap_or_else(|error| panic!("reading directory entry failed: {error}"))
            .path();
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

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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
permissions = ["administration:write", "administration:read", "merge"]

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
