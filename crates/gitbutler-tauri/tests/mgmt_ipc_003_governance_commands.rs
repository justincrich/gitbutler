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

/// Governance commands registered through the shared governance command rows.
const GOVERNANCE_COMMANDS: &[&str] = &[
    "perm_list",
    "perm_grant",
    "perm_revoke",
    "group_create",
    "group_grant",
    "group_revoke",
    "group_add_member",
    "group_remove_member",
    "group_delete",
    "group_list",
    "branch_gates_read",
    "branch_gates_update",
    "governance_status_read",
    "governance_principals_list",
    "governance_pending",
    "governance_commit",
    "principal_kind_read",
    "principal_kind_update",
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

    let error = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("rust-implementer")),
        ],
        || {
            invoke_err(
                &webview,
                "agent_perm_grant",
                grant_payload(&project_id, "rust-implementer", ["administration:write"]),
            )
        },
    )?;

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
fn branch_gates_desktop_identity_uses_principal_authorization_path() -> anyhow::Result<()> {
    let (repo, _tmp) = governance_api_repo(true);
    let project_id = project_id_for(&repo)?;

    let admin_app = governance_app(desktop_session_with_login("admin"))?;
    let admin_webview = governance_webview(&admin_app)?;
    temp_env::with_var("BUT_AGENT_HANDLE", None::<&str>, || {
        invoke_ok(
            &admin_webview,
            "branch_gates_read",
            json!({ "projectId": project_id, "targetRef": TARGET_REF }),
        )
    })?;

    let admin_update = temp_env::with_var("BUT_AGENT_HANDLE", Some("rust-implementer"), || {
        invoke_ok(
            &admin_webview,
            "branch_gates_update",
            json!({
                "projectId": project_id,
                "targetRef": TARGET_REF,
                "branch": "main",
                "protection": { "protected": false }
            }),
        )
    })?;
    assert!(
        admin_update
            .get("branches")
            .and_then(Value::as_array)
            .and_then(|branches| branches
                .iter()
                .find(|branch| { branch.get("name").and_then(Value::as_str) == Some("main") }))
            .and_then(|branch| branch.get("protected"))
            .and_then(Value::as_bool)
            == Some(false),
        "non-admin BUT_AGENT_HANDLE must not shadow desktop principal admin authority"
    );

    let denied_app = governance_app(desktop_session_with_login("rust-implementer"))?;
    let denied_webview = governance_webview(&denied_app)?;
    let denial = temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        invoke_err(
            &denied_webview,
            "branch_gates_update",
            json!({
                "projectId": project_id,
                "targetRef": TARGET_REF,
                "branch": "release",
                "protection": { "protected": true }
            }),
        )
    })?;
    assert_perm_denied_with_hint(&denial, "administration:write");

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
        "the IPC harness must invoke exactly the governance commands in the contract"
    );
    assert!(
        worktree_permissions(&repo)?.contains("reviews:write"),
        "perm_grant IPC call must reach the real mutation path"
    );
    Ok(())
}

#[test]
#[serial_test::serial]
fn group_revoke_removes_only_requested_group_authority() -> anyhow::Result<()> {
    let (repo, _tmp) = governance_api_repo(true);
    let project_id = project_id_for(&repo)?;
    let app = governance_app(test_desktop_session())?;
    let webview = governance_webview(&app)?;

    let response = temp_env::with_var("BUT_AGENT_HANDLE", None::<&str>, || {
        invoke_ok(
            &webview,
            "group_revoke",
            json!({
                "projectId": project_id,
                "targetRef": TARGET_REF,
                "group": "eng",
                "authorities": ["contents:write"]
            }),
        )
    })?;

    assert_eq!(response.get("group").and_then(Value::as_str), Some("eng"));
    assert_eq!(
        response.get("authorities").and_then(Value::as_array),
        Some(&vec![Value::String("contents:write".to_owned())]),
        "group_revoke must report the requested authority token"
    );

    let permissions = worktree_permissions(&repo)?;
    let eng = group_block(&permissions, "eng")?;
    assert!(
        !eng.contains("contents:write"),
        "group_revoke must remove only the requested direct authority"
    );
    assert!(
        eng.contains("reviews:write"),
        "group_revoke must preserve unrelated direct group authorities"
    );
    assert!(
        eng.contains("alice") && eng.contains("bob"),
        "group_revoke must preserve group members"
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
                || name.starts_with("allow-principal_kind_")
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

/// AC-5 / TC-8: the desktop `principal_kind_update` bus invoke (BUT_AGENT_HANDLE
/// unset) writes `kind="agent"` via the DesktopSessionState fleet-owner shim,
/// WITHOUT consulting `BUT_AGENT_HANDLE` (which is unset on desktop).
#[test]
#[serial_test::serial]
fn principal_kind_update_fleet_owner_writes_kind_without_agent_handle() -> anyhow::Result<()> {
    let (repo, _tmp) = governance_api_repo(true);
    let project_id = project_id_for(&repo)?;
    let app = governance_app(test_desktop_session())?;
    let webview = governance_webview(&app)?;

    let response = temp_env::with_var("BUT_AGENT_HANDLE", None::<&str>, || {
        invoke_ok(
            &webview,
            "principal_kind_update",
            json!({
                "projectId": project_id,
                "targetRef": TARGET_REF,
                "principal": "rust-implementer",
                "kind": "agent"
            }),
        )
    })?;

    // The desktop write returns the ref-pin caveat (inert until committed).
    assert_eq!(
        response.get("caveat").and_then(Value::as_str),
        Some("takes effect once committed to the target branch"),
        "principal_kind_update must carry the ref-pin caveat: {response:?}"
    );

    // The staged kind landed in the working-tree permissions.toml for rust-implementer.
    let staged = worktree_permissions(&repo)?;
    let rust_implementer = principal_block(&staged, "rust-implementer")?;
    assert!(
        rust_implementer.contains(r#"kind = "agent""#),
        "principal_kind_update must stage kind=\"agent\" for rust-implementer via the fleet-owner path: {rust_implementer}"
    );

    // principal_kind_read rides the but-api #[but_api(napi)]-generated module on core:default.
    let list = temp_env::with_var("BUT_AGENT_HANDLE", None::<&str>, || {
        invoke_ok(
            &webview,
            "principal_kind_read",
            json!({ "projectId": project_id, "targetRef": TARGET_REF }),
        )
    })?;
    let principals = list
        .get("principals")
        .and_then(Value::as_array)
        .expect("principal_kind_read must return a principals array");
    assert!(
        principals
            .iter()
            .any(|entry| entry.get("principalId").and_then(Value::as_str)
                == Some("rust-implementer")),
        "principal_kind_read list must include rust-implementer: {principals:?}"
    );

    println!(
        "principal_kind_update resolved the fleet-owner via DesktopSessionState (no BUT_AGENT_HANDLE)"
    );
    println!("principal_kind_update wrote kind=\"agent\" for rust-implementer to the working tree");
    println!("principal_kind_read returned the kind list on the bus");
    Ok(())
}

/// AC-5 / TC-8: an agent-env invoke lacking administration:write is denied
/// `perm.denied` naming `administration:write` and writes nothing.
///
/// Mirrors `mgmt_unauthorized_agent_config_command_denied` for `perm_grant`.
/// Uses `agent_perm_grant` as the proxy for the agent-env kind write — the
/// `principal_kind_update` desktop command always resolves the fleet-owner
/// (BUT_AGENT_HANDLE unset on desktop), so the agent-env denial path is
/// structurally identical to the existing `perm_grant` agent-env proof: the
/// admin gate at the but-api boundary catches it.
#[test]
#[serial_test::serial]
fn agent_env_admin_gate_denies_non_admin_kind_write_at_but_api_boundary() -> anyhow::Result<()> {
    // Prove the admin gate at the but-api boundary denies a non-admin kind write
    // (the desktop wrapper always resolves the fleet-owner; the agent-env path
    // reaches the but-api `principal_kind_update_with_repo` which composes
    // `enforce_administration_write_gate`). This is the same denial shape the
    // existing `mgmt_unauthorized_agent_config_command_denied` test exercises
    // for `perm_grant`; the kind writer reuses the exact same guard.
    use but_api::legacy::{
        config_mutate::classify_error, governance::principal_kind_update_with_repo,
    };

    let (repo, _tmp) = governance_api_repo(true);
    let before = worktree_permissions_bytes(&repo)?;

    let result = temp_env::with_var("BUT_AGENT_HANDLE", Some("rust-implementer"), || {
        principal_kind_update_with_repo(&repo, TARGET_REF, "rust-reviewer", "agent")
    });

    let error = classify_error(&result.expect_err("non-admin kind write must err"))
        .expect("non-admin denial must classify");
    assert_eq!(
        error.code, "perm.denied",
        "agent-env non-admin principal_kind_update must be denied perm.denied"
    );
    assert!(
        error.message.contains("administration:write"),
        "denial must name the missing administration:write authority"
    );
    assert_eq!(
        worktree_permissions_bytes(&repo)?,
        before,
        "denied non-admin kind write must leave permissions.toml byte-identical"
    );

    println!(
        "agent-env non-admin principal_kind_update denied perm.denied naming administration:write"
    );
    println!("working-tree permissions.toml byte-identical on the denial path");
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
            command: "group_revoke",
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
        InvocationCase {
            command: "governance_principals_list",
            payload: json!({ "projectId": project_id, "targetRef": TARGET_REF }),
        },
        InvocationCase {
            command: "governance_pending",
            payload: json!({ "projectId": project_id, "targetRef": TARGET_REF }),
        },
        InvocationCase {
            command: "governance_commit",
            payload: json!({ "projectId": project_id, "targetRef": TARGET_REF }),
        },
        InvocationCase {
            command: "principal_kind_read",
            payload: json!({ "projectId": project_id, "targetRef": TARGET_REF }),
        },
        InvocationCase {
            command: "principal_kind_update",
            payload: json!({
                "projectId": project_id,
                "targetRef": TARGET_REF,
                "principal": "rust-implementer",
                "kind": "agent"
            }),
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

[[group]]
name = "eng"
permissions = ["contents:write", "reviews:write"]
members = ["alice", "bob"]
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
    desktop_session_with_login("admin")
}

fn desktop_session_with_login(login: &str) -> gitbutler_tauri::governance::TestDesktopSession {
    gitbutler_tauri::governance::TestDesktopSession {
        user_id: 42,
        login: Some(login.to_owned()),
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

fn group_block<'a>(toml: &'a str, name: &str) -> anyhow::Result<&'a str> {
    let marker = format!(r#"name = "{name}""#);
    toml.split("[[group]]")
        .skip(1)
        .find(|block| block.contains(&marker))
        .ok_or_else(|| anyhow::anyhow!("expected [[group]] block with {marker}"))
}

fn principal_block<'a>(toml: &'a str, id: &str) -> anyhow::Result<&'a str> {
    let marker = format!(r#"id = "{id}""#);
    toml.split("[[principal]]")
        .skip(1)
        .find(|block| block.contains(&marker))
        .ok_or_else(|| anyhow::anyhow!("expected [[principal]] block with {marker}"))
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
