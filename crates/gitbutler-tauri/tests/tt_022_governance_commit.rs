//! tt-022 governance renderer contract command proofs.

use anyhow::Context as _;
use serde_json::{Value, json};
use tauri::{WebviewWindowBuilder, ipc::InvokeBody, webview::InvokeRequest};

const TARGET_REF: &str = "refs/remotes/origin/main";

#[test]
fn governance_commit_commits_only_pending_governance_files() -> anyhow::Result<()> {
    let (repo, _tmp) = governance_repo();
    but_testsupport::invoke_bash(
        r#"
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "admin"
permissions = ["administration:write", "administration:read", "merge"]

[[principal]]
id = "rust-implementer"
permissions = ["contents:write", "administration:write"]
EOF
cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true

[[branch]]
name = "release"
protected = true
EOF
echo "must not be included" >unrelated.txt
"#,
        &repo,
    );

    let project_id = project_id_for(&repo)?;
    let app = governance_app()?;
    let webview = governance_webview(&app)?;
    let response = invoke(
        &webview,
        "governance_commit",
        json!({ "projectId": project_id, "targetRef": TARGET_REF }),
    )?;
    let response = response.unwrap_or_else(|error| panic!("governance_commit failed: {error:?}"));

    assert_eq!(
        response.get("message").and_then(Value::as_str),
        Some("chore: update governance config"),
        "the governance commit command must use the fixed renderer contract message"
    );
    assert_eq!(
        response.get("committedPaths").and_then(Value::as_array),
        Some(&vec![
            Value::String(".gitbutler/gates.toml".to_owned()),
            Value::String(".gitbutler/permissions.toml".to_owned()),
        ]),
        "the governance commit command must report only committed governance paths"
    );

    let committed = repo
        .find_reference(TARGET_REF)?
        .peel_to_commit()
        .context("target ref must point at the new governance commit")?;
    assert_eq!(
        committed.message()?.title,
        "chore: update governance config",
        "target ref commit message must match the renderer contract"
    );
    assert!(
        committed_blob(&repo, ".gitbutler/permissions.toml")?.contains("administration:write"),
        "pending permissions.toml must be committed to the target ref"
    );
    assert!(
        committed_blob(&repo, ".gitbutler/gates.toml")?.contains("release"),
        "pending gates.toml must be committed to the target ref"
    );
    assert!(
        committed_blob(&repo, "unrelated.txt").is_err(),
        "unrelated worktree files must not be included in the governance commit"
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

fn committed_blob(repo: &gix::Repository, path: &str) -> anyhow::Result<String> {
    use gix::bstr::ByteSlice as _;

    let commit = repo.find_reference(TARGET_REF)?.peel_to_commit()?;
    let tree = commit.tree()?;
    let entry = tree
        .lookup_entry_by_path(gix::path::from_bstr(path.as_bytes().as_bstr()))?
        .with_context(|| format!("{path} must exist in committed tree"))?;
    let data = entry.object()?.into_blob().data.clone();
    Ok(String::from_utf8(data)?)
}
