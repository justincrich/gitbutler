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
const MERGE_GATE: &str = "crates/but-api/src/legacy/merge_gate.rs";
const CONFIG_MUTATE: &str = "crates/but-api/src/legacy/config_mutate.rs";
const GOVERNANCE: &str = "crates/but-api/src/legacy/governance.rs";
const FORGE_GUARD: &str = "crates/but-api/src/legacy/forge.rs";
const ENFORCEMENT_PATHS: &[&str] = &[
    AUTHZ_AUTHORIZE,
    AUTHZ_CONFIG,
    COMMIT_GATE,
    MERGE_GATE,
    CONFIG_MUTATE,
    GOVERNANCE,
    FORGE_GUARD,
];
const SPRINT_02_ENFORCEMENT_PATHS: &[&str] = &[MERGE_GATE, CONFIG_MUTATE];

// --- STEER-010: closed-catalog + route-table coverage build-gates -----------
//
// These build-gates are ADDITIVE to the honesty-grep suite above. They
// prove (a) the menu module's command/effect text is closed-catalog
// &'static str (invariant §9.2) — no format!, interpolation, or
// config-sourced text — and (b) the ROUTE_AUTHORITY_TABLE exists and is
// consulted by gate sites.

/// Pattern: menu.rs must NOT contain format!/to_string/to_owned for
/// command/effect/do_not text. The closed-catalog assertion covers ONLY the
/// steering fields (command/effect in menu.rs) — NOT message/unmet[] which
/// already interpolate config strings (R15 accepted-leak).
const CLOSED_CATALOG_VIOLATION_PATTERN: &str =
    r#"format!\s*\(|\.to_owned\(\)|\.to_string\(\)|String::from\(\s*""#;

/// Positive control: route.rs must have a non-empty ROUTE_AUTHORITY_TABLE.
const ROUTE_TABLE_EXISTS_PATTERN: &str = r"ROUTE_AUTHORITY_TABLE\s*:";

/// Gate sites must consult the route table (via Route:: variants or
/// required_authority() calls). Scoped to the five actual gate sites
/// (excludes but-authz internals that don't consume Route).
const ROUTE_TABLE_CONSUMED_PATTERN: &str = r"Route::|required_authority\s*\(";

const MENU_SRC: &str = "crates/but-authz/src/menu.rs";
const ROUTE_SRC: &str = "crates/but-authz/src/route.rs";

/// Gate sites that consume the route table (excludes authz internals).
const GATE_SITE_PATHS: &[&str] = &[
    COMMIT_GATE,
    MERGE_GATE,
    CONFIG_MUTATE,
    GOVERNANCE,
    FORGE_GUARD,
];

// --- STEER-008: non-enforced agent-priming reference primer -----------------
//
// The primer is L2 reference material (UC-STEER-05, Stance 6): a harness MAY
// adopt it, but the engine (but-authz / but-api) MUST NOT depend on it for
// correctness. The assertions below prove (a) the doc carries the four
// required claims, (b) no engine source tree references it, and (c) the
// non-dependence grep has teeth (a seeded violating fixture is detected).
const PRIMER_DOC: &str = "crates/but/governance-denial-primer.md";
// Engine source trees the non-dependence grep scans.
const ENGINE_SOURCE_TREES: &[&str] = &["crates/but-authz/src", "crates/but-api/src"];
// A reference to the primer that would indicate an engine code path depends on
// it: the primer filename (include_str! / path branch) or one of its
// distinctive header phrases.
const PRIMER_REFERENCE_PATTERN: &str =
    r#"governance-denial-primer|options, not orders|denials are redirects, not"#;

#[test]
fn invariant_build_gates() -> anyhow::Result<()> {
    let workspace_root = workspace_root()?;
    // The no-role-name / no-human-vs-AI invariants apply to EVERY enforcement
    // path, including the merge gate, admin-write guard, and forge boundary
    // guards -- not just the commit-gate path. Functional `Authority` is the
    // only axis any gate may branch on.
    assert!(
        ENFORCEMENT_PATHS.len() >= 5,
        "Sprint-02 enforcement coverage must include the Sprint-01a trio plus merge and admin-write surfaces"
    );
    assert_paths_exist_and_non_empty(&workspace_root, ENFORCEMENT_PATHS)?;

    assert_grep_has_no_matches(
        "role-preset branch invariant",
        &workspace_root,
        ROLE_BRANCH_PATTERN,
        ENFORCEMENT_PATHS,
    )?;
    assert_grep_has_no_matches(
        "human-vs-AI or role-label branch invariant",
        &workspace_root,
        HUMAN_OR_LABEL_BRANCH_PATTERN,
        ENFORCEMENT_PATHS,
    )?;
    assert_grep_has_matches(
        "commit gate must use the but-authz Authority axis",
        &workspace_root,
        AUTHORITY_POSITIVE_PATTERN,
        &[COMMIT_GATE],
    )?;
    assert_grep_has_matches(
        "merge gate must use the but-authz Authority axis",
        &workspace_root,
        AUTHORITY_POSITIVE_PATTERN,
        &[MERGE_GATE],
    )?;
    assert_grep_has_matches(
        "admin-write gate must use the but-authz Authority axis",
        &workspace_root,
        AUTHORITY_POSITIVE_PATTERN,
        &[CONFIG_MUTATE],
    )?;
    assert_grep_has_matches(
        "governance boundary must use the but-authz Authority axis",
        &workspace_root,
        AUTHORITY_POSITIVE_PATTERN,
        &[GOVERNANCE],
    )?;
    assert_grep_has_no_matches(
        "commit gate must not use GitButler Permission as authz carrier",
        &workspace_root,
        PERMISSION_CARRIER_PATTERN,
        &[COMMIT_GATE],
    )?;
    assert_grep_has_no_matches(
        "Sprint-02 gates must not use GitButler Permission as authz carrier",
        &workspace_root,
        PERMISSION_CARRIER_PATTERN,
        SPRINT_02_ENFORCEMENT_PATHS,
    )?;

    // --- STEER-010: closed-catalog + route-table coverage ------------------
    assert_paths_exist_and_non_empty(&workspace_root, &[MENU_SRC, ROUTE_SRC])?;

    assert_grep_has_no_matches(
        "closed-catalog invariant (menu.rs command/effect must be &'static str, no format!/interpolation)",
        &workspace_root,
        CLOSED_CATALOG_VIOLATION_PATTERN,
        &[MENU_SRC],
    )?;
    assert_grep_has_matches(
        "ROUTE_AUTHORITY_TABLE must be non-empty in route.rs",
        &workspace_root,
        ROUTE_TABLE_EXISTS_PATTERN,
        &[ROUTE_SRC],
    )?;
    assert_grep_has_matches(
        "gate sites must consult the route table (Route:: or required_authority)",
        &workspace_root,
        ROUTE_TABLE_CONSUMED_PATTERN,
        GATE_SITE_PATHS,
    )?;

    assert_seeded_controls_fire()?;

    Ok(())
}

// ===========================================================================
// STEER-008: non-enforced agent-priming reference primer (UC-STEER-05)
// ===========================================================================
// These four build-gates are ADDITIVE to the honesty-grep suite above. They
// prove the primer doc exists with the required content (AC-1/3/4) and that no
// engine code path depends on it (AC-2, Stance 6). They never weaken or
// relocate a shipped honesty-grep pattern.

#[test]
fn steer_primer_contains_required_statements() -> anyhow::Result<()> {
    let root = workspace_root()?;
    let primer = read_primer(&root)?;
    // AC-1: the four literal claims.
    assert_contains(&primer, "redirect", "AC-1 denials-are-redirects")?;
    assert_contains(&primer, "options, not orders", "AC-1 affordances")?;
    assert_contains(&primer, "bypass", "AC-1 no-bypass")?;
    assert_contains(&primer, "operator_required", "AC-1 class/do_not contract")?;
    assert_contains(&primer, "stop", "AC-1 stop-on-operator_required")?;
    Ok(())
}

#[test]
fn steer_primer_engine_independent() -> anyhow::Result<()> {
    let root = workspace_root()?;
    // AC-2 (Stance 6): the engine source trees MUST NOT reference the primer —
    // no include_str!, no path branch, no dependence on primer content.
    assert_grep_has_no_matches(
        "primer non-dependence invariant (engine must not reference the primer)",
        &root,
        PRIMER_REFERENCE_PATTERN,
        ENGINE_SOURCE_TREES,
    )?;

    // Teeth: a deliberately injected primer dependence MUST be detected. This
    // proves the grep bites — a silent (always-zero) grep would be worthless.
    let temp_dir = TempDir::new().context("create seeded primer-dependence temp directory")?;
    let dependence_fixture = temp_dir.path().join("primer-dependence-violation.rs");
    fs::write(
        &dependence_fixture,
        r#"
fn violates_primer_independence() {
    let _primer = include_str!("../../but/governance-denial-primer.md");
}
"#,
    )
    .with_context(|| format!("write {}", dependence_fixture.display()))?;
    assert_grep_has_matches(
        "seeded primer-dependence control (the non-dependence grep has teeth)",
        temp_dir.path(),
        PRIMER_REFERENCE_PATTERN,
        &["primer-dependence-violation.rs"],
    )?;
    Ok(())
}

#[test]
fn steer_primer_goal_integrity_and_contract() -> anyhow::Result<()> {
    let root = workspace_root()?;
    let primer = read_primer(&root)?;
    // AC-3: goal integrity — choose the authorized_actions entry that serves
    // your actual task (affordances != orders).
    assert_contains(&primer, "serves your", "AC-3 goal integrity")?;
    // AC-3: the class/do_not contract — both classes documented.
    assert_contains(&primer, "class", "AC-3 class contract")?;
    assert_contains(&primer, "actor_correctable", "AC-3 actor_correctable class")?;
    assert_contains(&primer, "operator_required", "AC-3 operator_required class")?;
    assert_contains(&primer, "do_not", "AC-3 do_not contract framing")?;
    Ok(())
}

#[test]
fn steer_primer_marked_non_enforced() -> anyhow::Result<()> {
    let root = workspace_root()?;
    let primer = read_primer(&root)?;
    // AC-4: the primer carries an explicit non-enforced reference marker.
    assert_contains(&primer, "non-enforced", "AC-4 non-enforced marker")?;
    assert_contains(&primer, "reference", "AC-4 reference marker")?;
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

    let catalog_fixture = temp_dir.path().join("closed-catalog-violation.rs");
    fs::write(
        &catalog_fixture,
        r#"
fn violates_closed_catalog() -> AuthorizedAction {
    AuthorizedAction::new(
        &format!("but {}", "commit"),
        "dynamic command".to_string(),
    )
}
"#,
    )
    .with_context(|| format!("write {}", catalog_fixture.display()))?;
    assert_grep_has_matches(
        "seeded closed-catalog violation control",
        temp_dir.path(),
        CLOSED_CATALOG_VIOLATION_PATTERN,
        &["closed-catalog-violation.rs"],
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

// --- STEER-008 primer helpers ----------------------------------------------

fn read_primer(root: &Path) -> anyhow::Result<String> {
    let path = root.join(PRIMER_DOC);
    fs::read_to_string(&path)
        .with_context(|| format!("primer doc is missing or unreadable: {}", path.display()))
}

fn assert_contains(haystack: &str, needle: &str, label: &str) -> anyhow::Result<()> {
    if !haystack.contains(needle) {
        bail!("{label}: primer missing required substring {needle:?}");
    }
    Ok(())
}
