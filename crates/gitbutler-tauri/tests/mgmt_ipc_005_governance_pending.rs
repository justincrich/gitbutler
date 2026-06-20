//! IPC-005 pending governance read proofs.
//!
//! These tests exercise Tauri's real mock runtime command bus with
//! `tauri::test::get_ipc_response`; the command reads real committed target-ref
//! config and real working-tree config bytes.

use std::{fs, path::Path};

use anyhow::Context as _;
use serde_json::{Value, json};
use tauri::{WebviewWindowBuilder, ipc::InvokeBody, webview::InvokeRequest};

const MAIN_REF: &str = "refs/heads/main";
const TARGET_REF: &str = "refs/remotes/origin/main";
const PERMISSIONS_PATH: &str = ".gitbutler/permissions.toml";
const GATES_PATH: &str = ".gitbutler/gates.toml";

#[test]
#[serial_test::serial]
fn governance_pending_reports_uncommitted_grant_as_pending() -> anyhow::Result<()> {
    let (repo, _tmp) = governance_repo(
        r#"
[[principal]]
id = "dev"
permissions = ["contents:write"]
"#,
    );
    write_worktree_permissions(
        &repo,
        r#"
[[principal]]
id = "dev"
permissions = ["contents:write", "reviews:write"]
"#,
    )?;

    let response = invoke_pending(&repo)?;
    let dev = principal(&response, "dev")?;

    assert_eq!(
        dev.get("committedEffective"),
        Some(&json!(["contents:write"])),
        "committed effective set must exclude the uncommitted reviews:write grant"
    );
    assert_pending(dev, "reviews:write", "grant");
    assert!(
        response
            .get("pendingCount")
            .and_then(Value::as_u64)
            .is_some_and(|count| count >= 1),
        "pendingCount must report at least the uncommitted reviews:write grant: {response:?}"
    );
    Ok(())
}

#[test]
#[serial_test::serial]
fn governance_pending_clean_tree_reports_zero() -> anyhow::Result<()> {
    let (repo, _tmp) = governance_repo(
        r#"
[[principal]]
id = "dev"
permissions = ["contents:write"]
"#,
    );

    let response = invoke_pending(&repo)?;
    let dev = principal(&response, "dev")?;

    assert_eq!(
        response.get("pendingCount").and_then(Value::as_u64),
        Some(0),
        "clean working tree must report no pending governance tokens"
    );
    assert_no_pending(dev, "contents:write");
    assert_no_pending(dev, "reviews:write");
    Ok(())
}

#[test]
#[serial_test::serial]
fn governance_pending_reports_uncommitted_revoke_as_pending() -> anyhow::Result<()> {
    let (repo, _tmp) = governance_repo(
        r#"
[[principal]]
id = "dev"
permissions = ["contents:write", "reviews:write"]
"#,
    );
    write_worktree_permissions(
        &repo,
        r#"
[[principal]]
id = "dev"
permissions = ["contents:write"]
"#,
    )?;

    let response = invoke_pending(&repo)?;
    let dev = principal(&response, "dev")?;

    assert_eq!(
        dev.get("committedEffective"),
        Some(&json!(["contents:write", "reviews:write"])),
        "committed effective set must keep reviews:write until the revoke is committed"
    );
    assert_pending(dev, "reviews:write", "revoke");
    assert!(
        response
            .get("pendingCount")
            .and_then(Value::as_u64)
            .is_some_and(|count| count >= 1),
        "pendingCount must report at least the uncommitted reviews:write revoke: {response:?}"
    );
    Ok(())
}

#[test]
#[serial_test::serial]
fn governance_pending_read_is_readonly_no_optimistic_enforcement() -> anyhow::Result<()> {
    let (repo, _tmp) = governance_repo(
        r#"
[[principal]]
id = "dev"
permissions = ["contents:write"]
"#,
    );
    write_worktree_permissions(
        &repo,
        r#"
[[principal]]
id = "dev"
permissions = ["contents:write", "reviews:write"]
"#,
    )?;
    let committed_permissions_before = committed_blob(&repo, PERMISSIONS_PATH)?;
    let committed_gates_before = committed_blob(&repo, GATES_PATH)?;
    let worktree_permissions_before = worktree_blob(&repo, PERMISSIONS_PATH)?;
    let worktree_gates_before = worktree_blob(&repo, GATES_PATH)?;

    let response = invoke_pending(&repo)?;
    let dev = principal(&response, "dev")?;
    assert_pending(dev, "reviews:write", "grant");

    let committed = but_authz::load_governance_config(&repo, TARGET_REF)?;
    let principal = but_authz::Principal::new(
        but_authz::PrincipalId::new("dev"),
        but_authz::AuthoritySet::empty(),
        std::iter::empty(),
    );
    let denial =
        match but_authz::authorize(&principal, but_authz::Authority::ReviewsWrite, &committed) {
            Ok(()) => anyhow::bail!(
                "committed target-ref config must deny uncommitted reviews:write grant"
            ),
            Err(denial) => denial,
        };
    assert_eq!(denial.code, "perm.denied");
    assert_eq!(
        committed_blob(&repo, PERMISSIONS_PATH)?,
        committed_permissions_before
    );
    assert_eq!(committed_blob(&repo, GATES_PATH)?, committed_gates_before);
    assert_eq!(
        worktree_blob(&repo, PERMISSIONS_PATH)?,
        worktree_permissions_before
    );
    assert_eq!(worktree_blob(&repo, GATES_PATH)?, worktree_gates_before);
    Ok(())
}

#[test]
#[serial_test::serial]
fn governance_pending_malformed_worktree_fails_closed() -> anyhow::Result<()> {
    let (repo, _tmp) = governance_repo(
        r#"
[[principal]]
id = "dev"
permissions = ["contents:write"]
"#,
    );
    write_worktree_permissions(&repo, "[[principal]\nid = \"dev\"\n")?;

    let error = invoke_pending_err(&repo)?;
    assert_eq!(
        error.get("code").and_then(Value::as_str),
        Some("config.invalid"),
        "malformed working-tree permissions.toml must fail closed as config.invalid: {error:?}"
    );
    assert!(
        error
            .get("remediation_hint")
            .and_then(Value::as_str)
            .is_some_and(|hint| !hint.is_empty()),
        "config.invalid transport error must carry a non-empty remediation_hint: {error:?}"
    );
    Ok(())
}

fn invoke_pending(repo: &gix::Repository) -> anyhow::Result<Value> {
    invoke_pending_result(repo)?.map_err(|error| anyhow::anyhow!("unexpected IPC error: {error:?}"))
}

fn invoke_pending_err(repo: &gix::Repository) -> anyhow::Result<Value> {
    match invoke_pending_result(repo)? {
        Ok(value) => anyhow::bail!("expected governance_pending IPC error, got {value:?}"),
        Err(error) => Ok(error),
    }
}

fn invoke_pending_result(repo: &gix::Repository) -> anyhow::Result<Result<Value, Value>> {
    let project_id = project_id_for(repo)?;
    let app = governance_app()?;
    let webview = governance_webview(&app)?;
    invoke(
        &webview,
        "governance_pending",
        json!({ "projectId": project_id, "targetRef": TARGET_REF }),
    )
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
                .find(|principal| principal.get("id").and_then(Value::as_str) == Some(id))
        })
        .with_context(|| format!("response must include principal {id}: {response:?}"))
}

fn assert_pending(principal: &Value, token: &str, kind: &str) {
    let entry = token_entry(principal, token).unwrap_or_else(|| {
        panic!("principal must include token {token} in pending diff: {principal:?}")
    });
    assert_eq!(
        entry.get("pending").and_then(Value::as_bool),
        Some(true),
        "{token} must be marked pending: {principal:?}"
    );
    assert_eq!(
        entry.get("change").and_then(Value::as_str),
        Some(kind),
        "{token} must be marked as pending {kind}: {principal:?}"
    );
}

fn assert_no_pending(principal: &Value, token: &str) {
    if let Some(entry) = token_entry(principal, token) {
        assert_ne!(
            entry.get("pending").and_then(Value::as_bool),
            Some(true),
            "{token} must not be pending in clean tree: {principal:?}"
        );
    }
}

fn token_entry<'a>(principal: &'a Value, token: &str) -> Option<&'a Value> {
    principal
        .get("tokens")
        .and_then(Value::as_array)?
        .iter()
        .find(|entry| entry.get("authority").and_then(Value::as_str) == Some(token))
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

fn worktree_blob(repo: &gix::Repository, path: &str) -> anyhow::Result<Vec<u8>> {
    fs::read(worktree_path(repo, path)?).map_err(Into::into)
}

fn worktree_path(repo: &gix::Repository, path: &str) -> anyhow::Result<std::path::PathBuf> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow::anyhow!("test repository must be non-bare"))?;
    Ok(workdir.join(path))
}

fn committed_blob(repo: &gix::Repository, path: &str) -> anyhow::Result<Vec<u8>> {
    let mut reference = repo
        .find_reference(TARGET_REF)
        .with_context(|| format!("resolving target ref {TARGET_REF}"))?;
    let commit = reference
        .peel_to_commit()
        .with_context(|| format!("peeling {TARGET_REF} to a commit"))?;
    let tree = commit
        .tree()
        .with_context(|| format!("reading tree for {TARGET_REF}"))?;
    let entry = tree
        .lookup_entry_by_path(Path::new(path))
        .with_context(|| format!("looking up {path} in {TARGET_REF}"))?
        .with_context(|| format!("missing {path} in {TARGET_REF}"))?;
    let blob = repo
        .find_blob(entry.id())
        .with_context(|| format!("reading {path} blob at {TARGET_REF}"))?;
    Ok(blob.data.to_vec())
}

fn project_id_for(
    repo: &gix::Repository,
) -> anyhow::Result<but_ctx::ProjectHandleOrLegacyProjectId> {
    let handle = but_ctx::ProjectHandle::from_path(repo.git_dir())?;
    Ok(but_ctx::ProjectHandleOrLegacyProjectId::ProjectHandle(
        handle,
    ))
}
