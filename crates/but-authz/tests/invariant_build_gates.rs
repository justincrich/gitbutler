use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use anyhow::{Context, bail};
use but_testsupport::gix_testtools::tempfile::TempDir;

const ROLE_BRANCH_PATTERN: &str = r#"== *"(read|triage|write|maintain|admin)"|"(read|triage|write|maintain|admin)" *=>|match[^;]*\brole\b|\bfrom_role\("#;
const HUMAN_OR_LABEL_BRANCH_PATTERN: &str = r#"is_human|is_ai|== *"(human|implementer|reviewer|maintainer)"|"(human|implementer|reviewer|maintainer)" *=>"#;
const AUTHORITY_POSITIVE_PATTERN: &str =
    r#"but_authz::authorize|Authority::contains|but_authz::Authority"#;
const PERMISSION_CARRIER_PATTERN: &str =
    r#"write_permission\(|RepoExclusive|\bPermissions?\b *[:.][^=]"#;

const AUTHZ_AUTHORIZE: &str = "crates/but-authz/src/authorize.rs";
const AUTHZ_CONFIG: &str = "crates/but-authz/src/config.rs";
const COMMIT_GATE: &str = "crates/but-api/src/commit/gate.rs";

#[test]
fn invariant_build_gates() -> anyhow::Result<()> {
    let workspace_root = workspace_root()?;
    let enforcement_paths = [AUTHZ_AUTHORIZE, AUTHZ_CONFIG, COMMIT_GATE];
    assert_paths_exist_and_non_empty(&workspace_root, &enforcement_paths)?;

    assert_grep_has_no_matches(
        "role-preset branch invariant",
        &workspace_root,
        ROLE_BRANCH_PATTERN,
        &enforcement_paths,
    )?;
    assert_grep_has_no_matches(
        "human-vs-AI or role-label branch invariant",
        &workspace_root,
        HUMAN_OR_LABEL_BRANCH_PATTERN,
        &enforcement_paths,
    )?;
    assert_grep_has_matches(
        "commit gate must use the but-authz Authority axis",
        &workspace_root,
        AUTHORITY_POSITIVE_PATTERN,
        &[COMMIT_GATE],
    )?;
    assert_grep_has_no_matches(
        "commit gate must not use GitButler Permission as authz carrier",
        &workspace_root,
        PERMISSION_CARRIER_PATTERN,
        &[COMMIT_GATE],
    )?;

    assert_seeded_controls_fire()?;

    Ok(())
}

fn workspace_root() -> anyhow::Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("CARGO_MANIFEST_DIR must resolve from crates/but-authz to the workspace root")
}

fn assert_paths_exist_and_non_empty(root: &Path, relative_paths: &[&str]) -> anyhow::Result<()> {
    for relative_path in relative_paths {
        let path = root.join(relative_path);
        let metadata = fs::metadata(&path)
            .with_context(|| format!("required grep path is missing: {}", path.display()))?;
        if !metadata.is_file() {
            bail!("required grep path is not a file: {}", path.display());
        }
        if metadata.len() == 0 {
            bail!("required grep path is empty: {}", path.display());
        }
    }
    Ok(())
}

fn assert_grep_has_no_matches(
    label: &str,
    cwd: &Path,
    pattern: &str,
    relative_paths: &[&str],
) -> anyhow::Result<()> {
    let output = grep(cwd, pattern, relative_paths)?;
    match output.status.code() {
        Some(1) => Ok(()),
        Some(0) => bail!(
            "{label} failed: grep found forbidden source matches\n{}",
            command_output(&output)
        ),
        Some(code) => bail!(
            "{label} failed: grep exited with status {code}\n{}",
            command_output(&output)
        ),
        None => bail!(
            "{label} failed: grep terminated by signal\n{}",
            command_output(&output)
        ),
    }
}

fn assert_grep_has_matches(
    label: &str,
    cwd: &Path,
    pattern: &str,
    relative_paths: &[&str],
) -> anyhow::Result<()> {
    let output = grep(cwd, pattern, relative_paths)?;
    if output.status.success() && !output.stdout.is_empty() {
        return Ok(());
    }

    bail!(
        "{label} failed: grep did not find the required structural match\n{}",
        command_output(&output)
    )
}

fn grep(cwd: &Path, pattern: &str, relative_paths: &[&str]) -> anyhow::Result<Output> {
    let mut command = Command::new("grep");
    command.current_dir(cwd).args(["-rEn", pattern]);
    command.args(relative_paths);
    command
        .output()
        .with_context(|| format!("failed to run grep from {}", cwd.display()))
}

fn assert_seeded_controls_fire() -> anyhow::Result<()> {
    let temp_dir = TempDir::new().context("create seeded violation temp directory")?;

    let role_fixture = temp_dir.path().join("role-branch-violation.rs");
    fs::write(
        &role_fixture,
        r#"
fn violates_role_branch(role: &str) {
    if role == "admin" {
        return;
    }
}
"#,
    )
    .with_context(|| format!("write {}", role_fixture.display()))?;
    assert_grep_has_matches(
        "seeded role-preset branch control",
        temp_dir.path(),
        ROLE_BRANCH_PATTERN,
        &["role-branch-violation.rs"],
    )?;

    let label_fixture = temp_dir.path().join("label-branch-violation.rs");
    fs::write(
        &label_fixture,
        r#"
fn violates_label_branch(actor: &Actor) {
    if actor.kind == "implementer" || is_human(actor) {
        return;
    }
}
"#,
    )
    .with_context(|| format!("write {}", label_fixture.display()))?;
    assert_grep_has_matches(
        "seeded human-vs-AI or role-label branch control",
        temp_dir.path(),
        HUMAN_OR_LABEL_BRANCH_PATTERN,
        &["label-branch-violation.rs"],
    )?;

    let carrier_fixture = temp_dir.path().join("permission-carrier-violation.rs");
    fs::write(
        &carrier_fixture,
        r#"
fn violates_permission_carrier(repo: &Repo) {
    let _guard = write_permission(repo);
    let _carrier = RepoExclusive;
    let _permission = Permission::Write;
}
"#,
    )
    .with_context(|| format!("write {}", carrier_fixture.display()))?;
    assert_grep_has_matches(
        "seeded Permission/write_permission carrier control",
        temp_dir.path(),
        PERMISSION_CARRIER_PATTERN,
        &["permission-carrier-violation.rs"],
    )?;

    Ok(())
}

fn command_output(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    format!(
        "status: {}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        output.status
    )
}
