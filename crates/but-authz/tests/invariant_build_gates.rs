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
    r#"but_authz::authorize|crate::authorize|Authority::contains|but_authz::Authority"#;
const PERMISSION_CARRIER_PATTERN: &str =
    r#"write_permission\(|RepoExclusive|\bPermissions?\b *[:.][^=]"#;

const AUTHZ_AUTHORIZE: &str = "crates/but-authz/src/authorize.rs";
const AUTHZ_CONFIG: &str = "crates/but-authz/src/config.rs";
const COMMIT_GATE: &str = "crates/but-api/src/commit/gate.rs";
// The ref-aware commit-gate enforcement primitive (the `authorize` call site)
// now lives in but-authz so lower-level commit producers can share it; the
// `but-api` commit/gate.rs above keeps only the Context-aware resolver + the
// CommitGateError payload. Both files are honesty-scanned.
const BUT_AUTHZ_COMMIT_GATE: &str = "crates/but-authz/src/commit_gate.rs";
const MERGE_GATE: &str = "crates/but-api/src/legacy/merge_gate.rs";
const CONFIG_MUTATE: &str = "crates/but-api/src/legacy/config_mutate.rs";
const GOVERNANCE: &str = "crates/but-api/src/legacy/governance.rs";
const FORGE_GUARD: &str = "crates/but-api/src/legacy/forge.rs";
const ENFORCEMENT_PATHS: &[&str] = &[
    AUTHZ_AUTHORIZE,
    AUTHZ_CONFIG,
    COMMIT_GATE,
    BUT_AUTHZ_COMMIT_GATE,
    MERGE_GATE,
    CONFIG_MUTATE,
    GOVERNANCE,
    FORGE_GUARD,
];
const SPRINT_02_ENFORCEMENT_PATHS: &[&str] = &[MERGE_GATE, CONFIG_MUTATE, FORGE_GUARD];

// --- LPR-009: safe-seam invariant build-gates --------------------------------
//
// The merge-gate code path (merge_gate.rs + review_requirement.rs) MUST NOT
// reference local_review_assignments, local_review_comments, or
// local_review_meta — the drive/gate separation enforced at build time.
const SAFE_SEAM_NO_READ: &str =
    r#"local_review_assignments|local_review_comments|local_review_meta"#;
const REVIEW_REQUIREMENT: &str = "crates/but-api/src/legacy/review_requirement.rs";
const SAFE_SEAM_GATE_PATHS: &[&str] = &[MERGE_GATE, REVIEW_REQUIREMENT];

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
    BUT_AUTHZ_COMMIT_GATE,
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

// ---------------------------------------------------------------------------
// STEER-010 — closed-catalog + table/affordance coverage honesty greps.
//
// These sit BESIDE the shipped no-role-preset (ROLE_BRANCH_PATTERN),
// no-human-vs-AI (HUMAN_OR_LABEL_BRANCH_PATTERN), positive-authorize
// (AUTHORITY_POSITIVE_PATTERN), and no-Permission (PERMISSION_CARRIER_PATTERN)
// patterns. They do NOT replace or weaken any shipped assertion.
// ---------------------------------------------------------------------------

/// Matches `format!` construction flowing into the NEW steering fields
/// (`authorized_actions` / `do_not`). The closed-catalog invariant (§9.2)
/// requires every command/effect in these fields to be a closed
/// `&'static str` constant — never `format!`, interpolated, config-sourced,
/// or model-generated. Scoped to the menu/authorize/denial construction
/// sites ONLY (not the whole `but-authz/src` tree) so the legitimate
/// `format!` in the R15 `message`/`remediation_hint` fields does not
/// false-positive.
const STEER_CLOSED_CATALOG_PATTERN: &str =
    r#"\bformat!\(.*\b(authorized_actions|do_not)\b|\b(authorized_actions|do_not)\b.*\bformat!\("#;

/// Matches `format!` in the R15 `message`/`remediation_hint` construction —
/// the ACCEPTED leak (R15 mitigates these separately). Asserting this pattern
/// HAS matches proves the closed-catalog grep is scoped correctly: the R15
/// fields DO use `format!` and the closed-catalog grep correctly EXCLUDES
/// them. If this assertion fails, either the R15 fields were accidentally
/// closed (a behaviour change outside this task's scope) or the grep paths
/// are wrong.
const R15_MESSAGE_INTERPOLATION_PATTERN: &str = r#"\b(message|remediation_hint)\b.*\bformat!\("#;

const AUTHZ_MENU: &str = "crates/but-authz/src/menu.rs";
const AUTHZ_DENIAL: &str = "crates/but-authz/src/denial.rs";
const AUTHZ_ROUTE: &str = "crates/but-authz/src/route.rs";

/// The closed-catalog grep scope: the exact menu/authorize/denial
/// construction sites STEER-003/004 own. NOT the whole `but-authz/src` tree
/// (the R15 `message`/`remediation_hint` construction in these same files
/// legitimately uses `format!` and must be in-scope for the R15 boundary
/// assertion but must NOT trip the closed-catalog grep).
const STEER_CLOSED_CATALOG_PATHS: &[&str] = &[AUTHZ_MENU, AUTHZ_AUTHORIZE, AUTHZ_DENIAL];

/// The 6 `Route` variants, hardcoded for text-based coverage checks. The
/// honesty-grep philosophy: assert over SOURCE TEXT, not runtime types, so
/// the gate catches violations in the text even if runtime behaviour is
/// coincidentally correct.
const STEER_ROUTE_VARIANTS: &[&str] = &[
    "Commit",
    "Merge",
    "ForgeReviewsWrite",
    "ForgeCommentsWrite",
    "ForgePullRequestsWrite",
    "Admin",
];

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
        &[BUT_AUTHZ_COMMIT_GATE],
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
    assert_grep_has_matches(
        "forge boundary must use the but-authz Authority axis",
        &workspace_root,
        AUTHORITY_POSITIVE_PATTERN,
        &[FORGE_GUARD],
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

    // --- LPR-009: safe-seam drive/gate separation --------------------------
    assert_grep_has_no_matches(
        "safe-seam invariant: gate path must not read drive tables",
        &workspace_root,
        SAFE_SEAM_NO_READ,
        SAFE_SEAM_GATE_PATHS,
    )?;
    // Positive control: gate path MUST reference local_review_verdicts
    // (the one table it is supposed to read). Without this the grep test
    // would pass vacuously if the gate path were empty or wrong.
    assert_grep_has_matches(
        "safe-seam invariant: gate path must reference local_review_verdicts (sanity control)",
        &workspace_root,
        r"local_review_verdicts",
        SAFE_SEAM_GATE_PATHS,
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

/// STEER-010 AC-1: closed-catalog grep scoped to the exact menu/authorize/
/// denial construction sites. Asserts NO `format!`/interpolation leaks into
/// the `authorized_actions`/`do_not` steering fields (the NEW fields), while
/// proving the R15 `message`/`remediation_hint` fields DO use `format!` (the
/// accepted leak) — demonstrating the grep is scoped correctly and does not
/// over-reach onto R15.
#[test]
fn steer_closed_catalog_no_interpolation_in_steering_fields() -> anyhow::Result<()> {
    let workspace_root = workspace_root()?;
    assert_paths_exist_and_non_empty(&workspace_root, STEER_CLOSED_CATALOG_PATHS)?;

    // Closed-catalog invariant: NO format!/interpolation leaks into the
    // authorized_actions or do_not steering fields. Every command/effect is
    // a closed &'static str catalog constant (§9.2).
    assert_grep_has_no_matches(
        "closed-catalog steering-fields invariant (no format! in authorized_actions/do_not)",
        &workspace_root,
        STEER_CLOSED_CATALOG_PATTERN,
        STEER_CLOSED_CATALOG_PATHS,
    )?;

    // R15 scope proof: format! IS present in message/remediation_hint (the
    // accepted R15 leak). This proves the closed-catalog grep is correctly
    // scoped — it does not over-reach onto the R15 fields. If this fails,
    // the R15 fields were accidentally closed (behaviour change) or the grep
    // paths are wrong.
    assert_grep_has_matches(
        "R15 message/remediation_hint interpolation present (closed-catalog scope proof)",
        &workspace_root,
        R15_MESSAGE_INTERPOLATION_PATTERN,
        STEER_CLOSED_CATALOG_PATHS,
    )?;

    Ok(())
}

/// STEER-010 AC-2: table/affordance coverage grep. Reads `route.rs` and
/// `menu.rs` as text and asserts:
///
/// 1. Every `Route::ALL` variant appears in `ROUTE_AUTHORITY_TABLE` (table
///    coverage — a gated route missing from the table is caught).
/// 2. Every `Route` variant has at least one entry in `AFFORDANCE_MAP`
///    (affordance coverage).
/// 3. No `AFFORDANCE_MAP` entry for an `Authority` predicate offers the
///    denied route as a candidate (no self-referencing affordance — offering
///    an un-held route would be a lying menu). The `BranchProtected`
///    predicate is the documented C5 succeeding-context exception (the caller
///    HOLDS the authority, just at a different ref).
#[test]
fn steer_table_affordance_coverage() -> anyhow::Result<()> {
    let workspace_root = workspace_root()?;
    assert_paths_exist_and_non_empty(&workspace_root, &[AUTHZ_ROUTE, AUTHZ_MENU])?;

    let route_src = fs::read_to_string(workspace_root.join(AUTHZ_ROUTE))
        .with_context(|| format!("read {AUTHZ_ROUTE}"))?;
    let menu_src = fs::read_to_string(workspace_root.join(AUTHZ_MENU))
        .with_context(|| format!("read {AUTHZ_MENU}"))?;

    // Isolate the const definitions (skip doc comments + doctests that may
    // reference the same tokens).
    let table_section = route_src
        .split("pub const ROUTE_AUTHORITY_TABLE:")
        .nth(1)
        .context("ROUTE_AUTHORITY_TABLE definition not found in route.rs")?;
    let affordance_section = menu_src
        .split("pub const AFFORDANCE_MAP:")
        .nth(1)
        .context("AFFORDANCE_MAP definition not found in menu.rs")?;

    // 1. Every Route::ALL variant appears in ROUTE_AUTHORITY_TABLE.
    for variant in STEER_ROUTE_VARIANTS {
        let token = format!("Route::{variant}");
        assert!(
            table_section.contains(&token),
            "Route::{variant} must appear in ROUTE_AUTHORITY_TABLE ({AUTHZ_ROUTE})"
        );
    }

    // 2. Every Route variant has at least one AFFORDANCE_MAP entry.
    for variant in STEER_ROUTE_VARIANTS {
        let token = format!("Route::{variant}");
        assert!(
            affordance_section.contains(&token),
            "Route::{variant} must have at least one AFFORDANCE_MAP entry ({AUTHZ_MENU})"
        );
    }

    // 3. No self-referencing affordance for Authority-predicate entries.
    //    For (Route::X, DenialPredicate::Authority, &[...]), no candidate
    //    may name Route::X. The caller LACKS X's authority, so offering it
    //    would be a lying menu. BranchProtected is the documented C5
    //    succeeding-context exception.
    for variant in STEER_ROUTE_VARIANTS {
        let route_token = format!("Route::{variant}");
        let mut search_from = 0;
        while let Some(rel_pos) = affordance_section[search_from..].find(&route_token) {
            let abs_pos = search_from + rel_pos;
            let window_end = (abs_pos + 100).min(affordance_section.len());
            let window_after = &affordance_section[abs_pos..window_end];

            if window_after.contains("DenialPredicate::Authority") {
                // This is an Authority entry for Route::VARIANT. Check the
                // candidate block (next 500 chars) for a self-referencing
                // affordance.
                let candidate_end = (abs_pos + 500).min(affordance_section.len());
                let candidate_window = &affordance_section[abs_pos..candidate_end];
                let self_ref = format!("Affordance::new({route_token}");
                assert!(
                    !candidate_window.contains(&self_ref),
                    "Authority-predicate AFFORDANCE_MAP entry for {route_token} \
                     must not offer the denied route as a candidate (lying menu)"
                );
            }
            search_from = abs_pos + route_token.len();
        }
    }

    Ok(())
}

/// STEER-010 AC-1 boundary/teeth control: proves the closed-catalog grep
/// has teeth on the NEW steering fields AND correctly excludes the R15
/// fields.
///
/// - R15 boundary: a `format!` in a `message`/`remediation_hint` construction
///   must NOT trip the closed-catalog grep (scope excludes R15).
/// - Teeth: a `format!` in an `authorized_actions`/`do_not` construction
///   MUST trip the closed-catalog grep (the gate bites on the new fields).
#[test]
fn steer_closed_catalog_r15_boundary_teeth_control() -> anyhow::Result<()> {
    let temp_dir = TempDir::new().context("create R15 boundary temp directory")?;

    // R15 boundary control: format! in message/remediation_hint must NOT
    // trip the closed-catalog grep (the scope correctly excludes R15).
    let r15_fixture = temp_dir.path().join("r15-message-interpolation.rs");
    fs::write(
        &r15_fixture,
        r#"fn build_denial(missing: &str) -> Denial {
    Denial {
        message: format!("action requires {}", missing),
        remediation_hint: format!("request a grant for {}", missing),
        authorized_actions: Vec::new(),
        do_not: None,
    }
}
"#,
    )
    .with_context(|| format!("write {}", r15_fixture.display()))?;
    assert_grep_has_no_matches(
        "R15 boundary: format! in message/remediation_hint must NOT trip closed-catalog grep",
        temp_dir.path(),
        STEER_CLOSED_CATALOG_PATTERN,
        &["r15-message-interpolation.rs"],
    )?;

    // Teeth control: format! in authorized_actions/do_not MUST trip the
    // closed-catalog grep (the gate has teeth on the new fields).
    let teeth_fixture = temp_dir.path().join("steering-field-interpolation.rs");
    fs::write(
        &teeth_fixture,
        r#"fn build_menu(name: &str) -> Denial {
    Denial {
        authorized_actions: vec![AuthorizedAction::new(format!("but {}", name), "effect")],
        do_not: Some(format!("do not retry as {}", name)),
        ..Default::default()
    }
}
"#,
    )
    .with_context(|| format!("write {}", teeth_fixture.display()))?;
    assert_grep_has_matches(
        "closed-catalog teeth: format! in authorized_actions/do_not MUST trip grep",
        temp_dir.path(),
        STEER_CLOSED_CATALOG_PATTERN,
        &["steering-field-interpolation.rs"],
    )?;

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

    let safe_seam_fixture = temp_dir.path().join("safe-seam-violation.rs");
    fs::write(
        &safe_seam_fixture,
        r#"
fn violates_safe_seam() {
    let _assignments = "local_review_assignments";
    let _comments = "local_review_comments";
    let _meta = "local_review_meta";
}
"#,
    )
    .with_context(|| format!("write {}", safe_seam_fixture.display()))?;
    assert_grep_has_matches(
        "seeded safe-seam violation control",
        temp_dir.path(),
        SAFE_SEAM_NO_READ,
        &["safe-seam-violation.rs"],
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
