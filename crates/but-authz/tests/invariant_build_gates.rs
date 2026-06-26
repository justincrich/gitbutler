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
const AUTHZ_REGISTRY: &str = "crates/but-authz/src/registry.rs";
const AUTHZ_PROCESS: &str = "crates/but-authz/src/process.rs";
const COMMIT_GATE: &str = "crates/but-api/src/commit/gate.rs";
const MERGE_GATE: &str = "crates/but-api/src/legacy/merge_gate.rs";
const CONFIG_MUTATE: &str = "crates/but-api/src/legacy/config_mutate.rs";
const GOVERNANCE: &str = "crates/but-api/src/legacy/governance.rs";
const FORGE_GUARD: &str = "crates/but-api/src/legacy/forge.rs";
const RULES: &str = "crates/but-api/src/legacy/rules.rs";
const ENFORCEMENT_PATHS: &[&str] = &[
    AUTHZ_AUTHORIZE,
    AUTHZ_CONFIG,
    AUTHZ_REGISTRY,
    AUTHZ_PROCESS,
    COMMIT_GATE,
    MERGE_GATE,
    CONFIG_MUTATE,
    GOVERNANCE,
    FORGE_GUARD,
];
const SPRINT_02_ENFORCEMENT_PATHS: &[&str] = &[MERGE_GATE, CONFIG_MUTATE];

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
const RESOLUTION_ORDER_DOC_LITERALS: &[&str] = &[
    "resolve_principal_with_runtime_registry",
    "but_authz::resolve_principal_with_registry",
    "BUT_AUTHZ_ALLOW_ENV_HANDLE",
    "Denial::unregistered",
];
const GOVERNANCE_RESOLUTION_ORDER_SURFACES: &[&str] = &[
    "governance_status_read",
    "branch_gates_read_with_repo",
    "group_list_with_repo",
    "perm_list_with_repo",
    "whoami_with_repo",
    "can_i_with_repo",
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

    assert_seeded_controls_fire()?;

    Ok(())
}

#[test]
fn test_enforcement_paths_extended() -> anyhow::Result<()> {
    let workspace_root = workspace_root()?;
    assert_eq!(
        ENFORCEMENT_PATHS.len(),
        9,
        "IDENT-020 enforcement coverage must include authorize/config plus registry/process hardening paths"
    );

    let source =
        fs::read_to_string(workspace_root.join("crates/but-authz/tests/invariant_build_gates.rs"))
            .context("read invariant build gate source")?;
    let enforcement_paths = source
        .split("const ENFORCEMENT_PATHS: &[&str] = &[")
        .nth(1)
        .and_then(|rest| rest.split("];").next())
        .context("ENFORCEMENT_PATHS source definition must be parseable")?;

    for token in [concat!("AUTHZ_", "REGISTRY"), concat!("AUTHZ_", "PROCESS")] {
        assert!(
            enforcement_paths.contains(token),
            "IDENT-020 ENFORCEMENT_PATHS source must include {token}"
        );
    }
    for line in [
        r#"const AUTHZ_REGISTRY: &str = "crates/but-authz/src/registry.rs";"#,
        r#"const AUTHZ_PROCESS: &str = "crates/but-authz/src/process.rs";"#,
    ] {
        assert!(
            source.contains(line),
            "IDENT-020 enforcement path constants must include {line}"
        );
    }

    assert_paths_exist_and_non_empty(&workspace_root, ENFORCEMENT_PATHS)?;
    Ok(())
}

#[test]
fn test_runtime_registry_wrapper_callsite_set_is_authoritative() -> anyhow::Result<()> {
    let workspace_root = workspace_root()?;
    let gate_src = fs::read_to_string(workspace_root.join(COMMIT_GATE))
        .with_context(|| format!("read {COMMIT_GATE}"))?;
    assert!(
        gate_src.contains(
            "but_authz::resolve_principal_with_registry(Some(&registry), cfg).map_err(Into::into)"
        ),
        "runtime wrapper must delegate to but_authz::resolve_principal_with_registry(Some(&registry), cfg)"
    );

    let expected = [
        (
            "crates/but-api/src/commit/gate.rs",
            "let principal = resolve_principal_with_runtime_registry(repo, &cfg)?;",
        ),
        (
            "crates/but-api/src/legacy/config_mutate.rs",
            "let principal = resolve_principal_with_runtime_registry(repo, &cfg)?;",
        ),
        (
            "crates/but-api/src/legacy/forge.rs",
            "let principal = resolve_principal_with_runtime_registry(repo, &cfg)?;",
        ),
        (
            "crates/but-api/src/legacy/governance.rs",
            "let authorities = match resolve_principal_with_runtime_registry(&repo, &config) {",
        ),
        (
            "crates/but-api/src/legacy/governance.rs",
            "let caller = resolve_principal_with_runtime_registry(repo, &config)?;",
        ),
        (
            "crates/but-api/src/legacy/governance.rs",
            "let caller = resolve_principal_with_runtime_registry(repo, &config)?;",
        ),
        (
            "crates/but-api/src/legacy/governance.rs",
            "let caller = resolve_principal_with_runtime_registry(repo, &config)?;",
        ),
        (
            "crates/but-api/src/legacy/governance.rs",
            "let caller = resolve_principal_with_runtime_registry(repo, &config)?;",
        ),
        (
            "crates/but-api/src/legacy/governance.rs",
            "let caller = resolve_principal_with_runtime_registry(repo, &config)?;",
        ),
        (
            "crates/but-api/src/legacy/merge_gate.rs",
            "let principal = resolve_principal_with_runtime_registry(&repo, &config.gov)?;",
        ),
        (
            "crates/but-api/src/legacy/rules.rs",
            "let caller = resolve_principal_with_runtime_registry(&repo, &config)?;",
        ),
    ];

    let mut actual = Vec::new();
    for relative_path in [
        COMMIT_GATE,
        CONFIG_MUTATE,
        FORGE_GUARD,
        GOVERNANCE,
        MERGE_GATE,
        "crates/but-api/src/legacy/rules.rs",
    ] {
        let source = fs::read_to_string(workspace_root.join(relative_path))
            .with_context(|| format!("read {relative_path}"))?;
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.contains("resolve_principal_with_runtime_registry(")
                && !trimmed.starts_with("pub(crate) fn ")
            {
                actual.push((relative_path, trimmed.to_owned()));
            }
        }
    }
    let expected = expected
        .into_iter()
        .map(|(path, line)| (path, line.to_owned()))
        .collect::<Vec<_>>();
    assert_eq!(
        actual, expected,
        "authoritative runtime wrapper callsite set changed"
    );
    Ok(())
}

#[test]
fn test_commit_gate_resolution_order_doc_precedes_function() -> anyhow::Result<()> {
    assert_resolution_order_docs_for_targets(&[(COMMIT_GATE, "enforce_commit_gate_for_target")])
}

#[test]
fn test_merge_gate_resolution_order_doc_precedes_function() -> anyhow::Result<()> {
    assert_resolution_order_docs_for_targets(&[(MERGE_GATE, "enforce_merge_gate")])
}

#[test]
fn test_governance_resolution_order_docs_precede_functions() -> anyhow::Result<()> {
    assert_eq!(
        GOVERNANCE_RESOLUTION_ORDER_SURFACES.len(),
        6,
        "IDENT-021 governance coverage must include exactly six target surfaces"
    );
    assert!(
        GOVERNANCE_RESOLUTION_ORDER_SURFACES.contains(&"whoami_with_repo")
            && GOVERNANCE_RESOLUTION_ORDER_SURFACES.contains(&"can_i_with_repo"),
        "IDENT-021 governance coverage must include whoami_with_repo and can_i_with_repo"
    );

    let targets = GOVERNANCE_RESOLUTION_ORDER_SURFACES
        .iter()
        .map(|function| (GOVERNANCE, *function))
        .collect::<Vec<_>>();
    assert_resolution_order_docs_for_targets(&targets)
}

#[test]
fn test_forge_resolution_order_doc_precedes_function() -> anyhow::Result<()> {
    assert_resolution_order_docs_for_targets(&[(FORGE_GUARD, "authorize_branch_action")])
}

#[test]
fn test_config_mutate_resolution_order_doc_precedes_function() -> anyhow::Result<()> {
    assert_resolution_order_docs_for_targets(&[(
        CONFIG_MUTATE,
        "enforce_administration_write_gate",
    )])
}

#[test]
fn test_rules_resolution_order_doc_precedes_function() -> anyhow::Result<()> {
    assert_resolution_order_docs_for_targets(&[(RULES, "list_workspace_rules_scoped_for_caller")])
}

#[test]
fn test_resolution_order_documented() -> anyhow::Result<()> {
    let mut targets = vec![
        (COMMIT_GATE, "enforce_commit_gate_for_target"),
        (MERGE_GATE, "enforce_merge_gate"),
        (FORGE_GUARD, "authorize_branch_action"),
        (CONFIG_MUTATE, "enforce_administration_write_gate"),
        (RULES, "list_workspace_rules_scoped_for_caller"),
    ];
    targets.extend(
        GOVERNANCE_RESOLUTION_ORDER_SURFACES
            .iter()
            .map(|function| (GOVERNANCE, *function)),
    );
    assert_eq!(
        targets.len(),
        11,
        "IDENT-021 must cover all 11 runtime-registry identity surfaces"
    );
    assert_resolution_order_docs_for_targets(&targets)
}

#[test]
fn test_but_agent_handle_env_reads_only_in_authorize() -> anyhow::Result<()> {
    let workspace_root = workspace_root()?;
    let governed_sources = [
        AUTHZ_CONFIG,
        "crates/but-authz/src/denial.rs",
        "crates/but-authz/src/lib.rs",
        "crates/but-authz/src/menu.rs",
        "crates/but-authz/src/process.rs",
        "crates/but-authz/src/registry.rs",
        AUTHZ_ROUTE,
        COMMIT_GATE,
        CONFIG_MUTATE,
        FORGE_GUARD,
        GOVERNANCE,
        MERGE_GATE,
        "crates/but-api/src/legacy/rules.rs",
    ];
    let mut direct_env_reads = Vec::new();
    for relative_path in governed_sources {
        let source = fs::read_to_string(workspace_root.join(relative_path))
            .with_context(|| format!("read {relative_path}"))?;
        for (line_index, line) in source.lines().enumerate() {
            let reads_env = line.contains("env::var") || line.contains("std::env::var");
            if line.contains("BUT_AGENT_HANDLE") && reads_env {
                direct_env_reads.push(format!("{relative_path}:{}:{line}", line_index + 1));
            }
        }
    }
    assert!(
        direct_env_reads.is_empty(),
        "direct_env_reads=0 outside authorize.rs; found:\n{}",
        direct_env_reads.join("\n")
    );
    Ok(())
}

#[test]
fn test_agents_path_exists() -> anyhow::Result<()> {
    let workspace_root = workspace_root()?;
    let config_src = fs::read_to_string(workspace_root.join(AUTHZ_CONFIG))
        .with_context(|| format!("read {AUTHZ_CONFIG}"))?;
    assert!(
        config_src.contains(r#"const AGENTS_PATH: &str = ".gitbutler/agents.toml";"#),
        "AGENTS_PATH must remain the preferred governance identity config path"
    );
    Ok(())
}

#[test]
fn test_permissions_path_deprecated() -> anyhow::Result<()> {
    let workspace_root = workspace_root()?;
    let config_src = fs::read_to_string(workspace_root.join(AUTHZ_CONFIG))
        .with_context(|| format!("read {AUTHZ_CONFIG}"))?;
    let permissions_line = config_src
        .lines()
        .position(|line| line.contains("const PERMISSIONS_PATH: &str"))
        .context("PERMISSIONS_PATH const must exist")?;
    let lines = config_src.lines().collect::<Vec<_>>();
    let previous_line = lines[..permissions_line]
        .iter()
        .rev()
        .find(|line| !line.trim().is_empty())
        .context("PERMISSIONS_PATH must have a preceding attribute line")?;
    assert!(
        previous_line.trim_start().starts_with("#[deprecated"),
        "#[deprecated] must be immediately above PERMISSIONS_PATH; found previous line {previous_line:?}"
    );
    Ok(())
}

#[test]
fn test_registry_ttl_process_identity_negative_controls_present() -> anyhow::Result<()> {
    let workspace_root = workspace_root()?;
    let registry_src =
        fs::read_to_string(workspace_root.join("crates/but-authz/tests/registry.rs"))
            .context("read registry tests")?;
    let process_src = fs::read_to_string(workspace_root.join("crates/but-authz/tests/process.rs"))
        .context("read process tests")?;
    let gate_swap_src =
        fs::read_to_string(workspace_root.join("crates/but-api/tests/gate_registry_swap.rs"))
            .context("read gate registry swap tests")?;
    let combined = format!("{registry_src}\n{process_src}\n{gate_swap_src}");

    for required in [
        "IDENT_001_same_pid_with_different_start_time_does_not_resolve",
        "IDENT_001_gc_keeps_expiry_boundary_and_drops_afterwards",
        "IDENT_002_nonexistent_pid_returns_error_that_names_pid",
        "expired_current_process_registry_entry_denied",
        "current_pid_wrong_start_time_denied_at_commit_gate",
        "wrong_pid_current_start_time_denied_at_commit_gate",
        "malformed_registry_propagates_instead_of_empty",
    ] {
        assert!(
            combined.contains(required),
            "registry/process identity coverage must include negative-control test {required}"
        );
    }

    let success_only_needles = [
        "registered_process_allowed",
        "register_resolve_round_trip",
        "write_then_load_round_trips",
    ];
    let has_success_coverage = success_only_needles
        .iter()
        .all(|needle| combined.contains(needle));
    let has_negative_coverage = [
        "_denied",
        "_does_not_resolve",
        "_returns_error",
        "_drops_afterwards",
        "_propagates_instead_of_empty",
    ]
    .iter()
    .all(|needle| combined.contains(needle));
    assert!(
        has_success_coverage && has_negative_coverage,
        "registry coverage must not be success-only; required negative controls are part of the invariant"
    );
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

fn assert_resolution_order_docs_for_targets(targets: &[(&str, &str)]) -> anyhow::Result<()> {
    let workspace_root = workspace_root()?;
    let mut failures = Vec::new();
    for (relative_path, function_name) in targets {
        let source = fs::read_to_string(workspace_root.join(relative_path))
            .with_context(|| format!("read {relative_path}"))?;
        let doc_block = match preceding_doc_block(&source, function_name) {
            Ok(doc_block) => doc_block,
            Err(error) => {
                failures.push(format!(
                    "{relative_path}::{function_name} missing preceding /// doc block: {error:#}"
                ));
                continue;
            }
        };
        for literal in RESOLUTION_ORDER_DOC_LITERALS {
            if !doc_block.contains(literal) {
                failures.push(format!(
                    "{relative_path}::{function_name} doc block missing {literal:?}"
                ));
            }
        }
    }

    if failures.is_empty() {
        Ok(())
    } else {
        bail!(
            "IDENT-021 resolution-order docs must name the runtime-registry resolver chain and denial order:\n{}",
            failures.join("\n")
        )
    }
}

fn preceding_doc_block(source: &str, function_name: &str) -> anyhow::Result<String> {
    let function_line = source
        .lines()
        .position(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with(&format!("pub fn {function_name}("))
                || trimmed.starts_with(&format!("pub async fn {function_name}("))
                || trimmed.starts_with(&format!("pub(crate) fn {function_name}("))
        })
        .with_context(|| format!("function {function_name} not found"))?;
    let lines = source.lines().collect::<Vec<_>>();
    let mut cursor = function_line;
    while cursor > 0 {
        let previous = lines[cursor - 1].trim_start();
        if previous.is_empty() || previous.starts_with("#[") {
            cursor -= 1;
            continue;
        }
        break;
    }

    let mut doc_lines = Vec::new();
    while cursor > 0 {
        let previous = lines[cursor - 1].trim_start();
        if let Some(doc) = previous.strip_prefix("///") {
            doc_lines.push(doc.trim_start());
            cursor -= 1;
        } else {
            break;
        }
    }
    doc_lines.reverse();
    if doc_lines.is_empty() {
        bail!("function {function_name} has no immediately preceding /// doc block");
    }
    Ok(doc_lines.join("\n"))
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
