//! tt-024 `principal_kind` governance IPC commands.
//!
//! Exercises the real Tauri mock-runtime command bus for the additive
//! `kind` descriptor on `[[principal]]` entries. The `kind` field is
//! enforcement-neutral (see `PrincipalWire::kind`); these commands are the
//! read/stage-write surface the governance UI uses, following the same
//! pending→commit flow as `perm_grant` (writes land in the working-tree
//! `.gitbutler/permissions.toml` and are committed later by `governance_commit`).

use anyhow::Context as _;
use serde_json::{Value, json};
use tauri::{WebviewWindowBuilder, ipc::InvokeBody, webview::InvokeRequest};

const TARGET_REF: &str = "refs/remotes/origin/main";
const PERMISSIONS_PATH: &str = ".gitbutler/permissions.toml";
/// Operator-facing caveat mirrored from `but_api::legacy::governance::REF_PIN_CAVEAT`.
const REF_PIN_CAVEAT: &str = "takes effect once committed to the target branch";

#[test]
fn get_principal_kind_returns_none_when_unset() -> anyhow::Result<()> {
    let (repo, _tmp) = governance_repo();
    let project_id = project_id_for(&repo)?;
    let app = governance_app()?;
    let webview = governance_webview(&app)?;

    let response = invoke_ok(
        &webview,
        "get_principal_kind",
        json!({ "projectId": project_id, "targetRef": TARGET_REF, "principal": "admin" }),
    )?;

    assert_eq!(
        response.get("principal").and_then(Value::as_str),
        Some("admin"),
        "get_principal_kind must echo the queried principal id"
    );
    assert!(
        response.get("kind").is_some_and(Value::is_null),
        "get_principal_kind must report null for a principal with no committed/pending kind"
    );
    Ok(())
}

#[test]
fn set_principal_kind_stages_kind_in_working_tree() -> anyhow::Result<()> {
    let (repo, _tmp) = governance_repo();
    let project_id = project_id_for(&repo)?;
    let app = governance_app()?;
    let webview = governance_webview(&app)?;

    let response = invoke_ok(
        &webview,
        "set_principal_kind",
        json!({
            "projectId": project_id,
            "targetRef": TARGET_REF,
            "principal": "admin",
            "kind": "agent"
        }),
    )?;

    assert_eq!(
        response.get("principal").and_then(Value::as_str),
        Some("admin"),
        "set_principal_kind must echo the written principal id"
    );
    assert_eq!(
        response.get("kind").and_then(Value::as_str),
        Some("agent"),
        "set_principal_kind must report the staged kind value"
    );
    assert_eq!(
        response.get("caveat").and_then(Value::as_str),
        Some(REF_PIN_CAVEAT),
        "set_principal_kind must carry the ref-pin caveat so operators know a commit is required"
    );

    let staged = worktree_permissions(&repo)?;
    assert!(
        staged.contains("kind = \"agent\""),
        "set_principal_kind must stage the kind into the working-tree permissions.toml: {staged}"
    );

    // The follow-up read must observe the just-staged value (effective read).
    let readback = invoke_ok(
        &webview,
        "get_principal_kind",
        json!({ "projectId": project_id, "targetRef": TARGET_REF, "principal": "admin" }),
    )?;
    assert_eq!(
        readback.get("kind").and_then(Value::as_str),
        Some("agent"),
        "get_principal_kind must observe the pending working-tree kind before commit"
    );
    Ok(())
}

#[test]
fn set_principal_kind_none_clears_the_field() -> anyhow::Result<()> {
    let (repo, _tmp) = governance_repo();
    let project_id = project_id_for(&repo)?;
    let app = governance_app()?;
    let webview = governance_webview(&app)?;

    invoke_ok(
        &webview,
        "set_principal_kind",
        json!({
            "projectId": project_id,
            "targetRef": TARGET_REF,
            "principal": "admin",
            "kind": "agent"
        }),
    )?;
    assert!(
        worktree_permissions(&repo)?.contains("kind = \"agent\""),
        "precondition: kind must be staged before clearing"
    );

    let response = invoke_ok(
        &webview,
        "set_principal_kind",
        json!({
            "projectId": project_id,
            "targetRef": TARGET_REF,
            "principal": "admin",
            "kind": null
        }),
    )?;
    assert!(
        response.get("kind").is_some_and(Value::is_null),
        "set_principal_kind must report null after clearing the kind"
    );

    let staged = worktree_permissions(&repo)?;
    assert!(
        !staged.contains("kind ="),
        "clearing kind must remove the field from the working-tree permissions.toml: {staged}"
    );
    Ok(())
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

fn invoke_ok(
    webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
    command: &str,
    payload: Value,
) -> anyhow::Result<Value> {
    match invoke(webview, command, payload)? {
        Ok(value) => Ok(value),
        Err(error) => anyhow::bail!("unexpected IPC error from {command}: {error:?}"),
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

fn governance_repo() -> (gix::Repository, tempfile::TempDir) {
    let tmp = tempfile::TempDir::new()
        .unwrap_or_else(|error| panic!("creating temp repository failed: {error}"));
    but_testsupport::invoke_bash_at_dir(
        r#"
git init --initial-branch=main
git config user.name "GitButler Test"
git config user.email "gitbutler@example.com"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "admin"
permissions = ["administration:write", "administration:read", "merge"]

[[principal]]
id = "rust-implementer"
permissions = ["contents:write"]
EOF
cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF
git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "governance config"
git update-ref refs/remotes/origin/main refs/heads/main
"#,
        tmp.path(),
    );
    let repo = gix::open(tmp.path())
        .unwrap_or_else(|error| panic!("opening {} failed: {error}", tmp.path().display()));
    (repo, tmp)
}

fn project_id_for(
    repo: &gix::Repository,
) -> anyhow::Result<but_ctx::ProjectHandleOrLegacyProjectId> {
    let handle = but_ctx::ProjectHandle::from_path(repo.git_dir())?;
    Ok(but_ctx::ProjectHandleOrLegacyProjectId::ProjectHandle(
        handle,
    ))
}

fn worktree_permissions(repo: &gix::Repository) -> anyhow::Result<String> {
    let path = repo
        .workdir()
        .ok_or_else(|| anyhow::anyhow!("test repository must be non-bare"))?
        .join(PERMISSIONS_PATH);
    std::fs::read_to_string(path)
        .with_context(|| format!("reading working-tree {PERMISSIONS_PATH}"))
}
