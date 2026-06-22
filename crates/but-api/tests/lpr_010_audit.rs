//! LPR-010 — SDK regen + N-API audit (R14) + honesty greps + drive-layer
//! integrity proofs (R22).
//!
//! This is the reviewer-owned audit/closeout lane for the LPR slice. It does
//! NOT add production logic — it verifies the LPR-001..009 implementations
//! satisfy the audit constraints:
//!
//! 1. **N-API audit (R14)** — the seven LPR verbs (`request_review`,
//!    `assign_reviewer`, `request_changes_review`, `post_comment`,
//!    `list_comments`, `resolve_thread`, `review_status`) are
//!    `#[but_api(napi)]`-decorated but-api fns (not Tauri-only commands).
//!    `#[but_api(napi)]` is the audited seam: every binding routes through
//!    `authorize_branch_action`, so the Electron lite app's N-API surface
//!    inherits the gate. R14 is satisfied iff there is NO parallel ungated
//!    N-API route.
//!
//! 2. **Honesty greps (T-LPR-022 / R23)** — the agent-authored tag is
//!    descriptive metadata on the read-only `ReviewStatus` DTO, NOT a key
//!    read by any gate-decision path. And the tag is sourced from the
//!    dedicated `local_review_meta(target, "opener_principal")` row (LPR-003),
//!    NEVER from a comment body — `__pr_meta__` is the reserved/rejected
//!    thread id (R23 negative control).
//!
//! 3. **Drive-layer integrity proofs (R22)** — T-LPR-043: a self-assignment
//!    (`assign_reviewer(reviewer == opener)`) is REJECTED with no row written
//!    (the drive layer cannot narrate a self-assigned reviewer as
//!    independently reviewed). T-LPR-044: an unauthorized self-resolve (by a
//!    non-author / non-assigned / non-`reviews:write` principal) is REJECTED
//!    and the thread stays unresolved (a cross-principal actor cannot forge
//!    an all-clear and suppress another party's remediation signal).
//!
//! No mocks — real but-api verbs + real but-db + real gix via
//! but_testsupport, mirroring the `forge_guard` / `local_review_*` test
//! idioms.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use anyhow::{Context as _, bail};

const FEAT_REF: &str = "refs/heads/feat";

// ============================================================================
// AC-4 / TC-6 (R14): N-API audit — the seven LPR verbs are #[but_api(napi)]
// ============================================================================

/// Every LPR review verb must be `#[but_api(napi)]`-decorated. R14 requires
/// consequential N-API routes to route through the gated but-api seam — a
/// bare `#[tauri::command]` (or no decoration at all) would be a parallel
/// ungated route and would fail this audit.
///
/// This test reuses the shipped honesty-grep discipline (a structural grep
/// over the but-api source) rather than a runtime introspection — the
/// `#[but_api(napi)]` macro is the audited attribute, and a structural grep
/// is the most direct proof.
#[test]
fn napi_audit_lpr_verbs_route_through_gated_but_api() -> anyhow::Result<()> {
    let root = workspace_root()?;
    let forge_path = root.join("crates/but-api/src/legacy/forge.rs");
    let src = fs::read_to_string(&forge_path)
        .with_context(|| format!("reading {}", forge_path.display()))?;

    let verbs = [
        "request_review",
        "assign_reviewer",
        "request_changes_review",
        "post_comment",
        "list_comments",
        "resolve_thread",
        "review_status",
    ];

    let lines: Vec<&str> = src.lines().collect();
    for verb in verbs {
        let mut found_gated = false;
        for (i, line) in lines.iter().enumerate() {
            if !line.trim_start().starts_with("#[but_api(napi)]") {
                continue;
            }
            // The `pub [async] fn <verb>` signature must appear within the
            // next few lines (allowing for `#[instrument(...)]` etc.).
            let mut signature_window = lines.iter().skip(i + 1).take(5);
            if signature_window.any(|succ| succ.contains("fn ") && succ.contains(verb)) {
                found_gated = true;
                break;
            }
        }
        assert!(
            found_gated,
            "R14 N-API audit: verb `{verb}` must be `#[but_api(napi)]`-decorated in \
             crates/but-api/src/legacy/forge.rs — a bare/Tauri-only command would be a \
             parallel ungated N-API route"
        );
    }
    Ok(())
}

// ============================================================================
// AC-2 / TC-3 (T-LPR-022): honesty grep — agent tag is NOT a gate-decision key
// ============================================================================

/// The gate-DECISION paths — where an enforcement key would actually be read.
/// This is a deliberately narrow subset of the shipped ENFORCEMENT_PATHS:
/// `forge.rs` is excluded because it carries `agent_authored: bool` as
/// descriptive metadata on the read-only `ReviewStatus` DTO (never read by
/// any gate). Scoping the grep to the decision paths is the honest test that
/// the tag has not leaked into gate logic.
const GATE_DECISION_PATHS: &[&str] = &[
    "crates/but-api/src/legacy/merge_gate.rs",
    "crates/but-api/src/commit/gate.rs",
];

/// Matches the `agent_authored` / `agent-authored` symbol — the descriptive
/// tag itself. If this matches inside a gate-decision path, the tag has
/// become an enforcement key (role-separation leaked into a label).
const AGENT_TAG_PATTERN: &str = r#"agent[-_]authored"#;

#[test]
fn agent_tag_not_an_enforcement_key() -> anyhow::Result<()> {
    let root = workspace_root()?;
    assert_paths_exist_and_non_empty(&root, GATE_DECISION_PATHS)?;
    assert_grep_has_no_matches(
        "agent-authored tag must not be a gate-decision key (T-LPR-022)",
        &root,
        AGENT_TAG_PATTERN,
        GATE_DECISION_PATHS,
    )
}

/// Positive control: the grep is connected — the tag IS present as
/// descriptive metadata on the read-only `ReviewStatus` DTO in forge.rs.
/// If this ever stops matching, the `agent_tag_not_an_enforcement_key` test
/// would become a disconnected no-op.
#[test]
fn agent_tag_is_descriptive_metadata_on_review_status() -> anyhow::Result<()> {
    let root = workspace_root()?;
    assert_grep_has_matches(
        "agent_authored must be present as descriptive metadata on ReviewStatus",
        &root,
        r#"pub\s+agent_authored:\s*bool"#,
        &["crates/but-api/src/legacy/forge.rs"],
    )
}

// ============================================================================
// AC-3 / TC-5 (R23): tag is sourced from local_review_meta, NOT comment body
// ============================================================================

/// The agent-tag source is the `local_review_meta(target, "opener_principal")`
/// row written by `request_review` (LPR-003). The opener's declared `kind` in
/// committed `permissions.toml` is read at the target ref to derive
/// `agent_authored` — never handle-resolution, never a comment body. This is
/// the R23 negative control: a comment-body sentinel (`__pr_meta__`) cannot
/// forge the tag.
#[test]
fn agent_tag_sourced_from_local_review_meta_not_comment_body() -> anyhow::Result<()> {
    let root = workspace_root()?;
    let forge = "crates/but-api/src/legacy/forge.rs";

    // Positive: the opener_principal is read from local_review_meta to derive
    // the tag.
    assert_grep_has_matches(
        "agent-tag derivation must read opener_principal from local_review_meta",
        &root,
        r#"local_review_meta\(\)\s*\.get\([^)]*opener_principal"#,
        &[forge],
    )?;

    // Positive: `__pr_meta__` is reserved/rejected — post_comment refuses it.
    // This is the load-bearing guard that prevents a comment-body sentinel
    // from forging the opener marker.
    assert_grep_has_matches(
        "post_comment must refuse the reserved __pr_meta__ thread_id",
        &root,
        r#"thread_id\s*!=\s*RESERVED_PR_META_THREAD"#,
        &[forge],
    )?;

    // Negative: nothing in forge.rs derives the tag from a comment body. The
    // tag is computed from the opener_principal row + the committed
    // permissions.toml `kind`, never from `local_review_comments`.
    assert_grep_has_no_matches(
        "agent tag must not be derived from a comment body",
        &root,
        r#"local_review_comments.*agent[-_]authored|agent[-_]authored.*local_review_comments"#,
        &[forge],
    )
}

// ============================================================================
// AC-6 / TC-8 (T-LPR-043 + T-LPR-044): drive-layer integrity proofs (R22)
// ============================================================================

/// T-LPR-043 — a self-assignment (`assign_reviewer(reviewer == opener)`) is
/// REJECTED at the but-api boundary with NO `(target, opener)` assignment row
/// written. R22: the drive layer cannot narrate a self-assigned reviewer as
/// independently reviewed.
#[tokio::test]
#[serial_test::serial]
async fn t_lpr_043_self_assignment_rejected_no_row_written() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_governed_repo();
    let ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();

    // `auth` opens the review — this writes the write-once
    // `local_review_meta(feat, "opener_principal") = "auth"` row. `auth` is
    // therefore the target branch author principal for R22.
    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("auth"))], async {
        but_api::legacy::forge::request_review(ctx.to_sync(), FEAT_REF.to_owned(), None).await
    })
    .await
    .context("request_review as `auth` must succeed to establish the opener principal")?;

    // `rev` holds reviews:write — the assignment authority. It attempts to
    // assign `auth` (the opener) as the reviewer — a self-assignment under
    // R22 — and must be REJECTED.
    let self_assign_err =
        match temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("rev"))], async {
            but_api::legacy::forge::assign_reviewer(
                ctx.to_sync(),
                FEAT_REF.to_owned(),
                "auth".to_owned(),
            )
            .await
        })
        .await
        {
            Ok(()) => bail!(
                "T-LPR-043: assign_reviewer(reviewer=auth == opener) must be REJECTED — \
             R22 distinct-from-author"
            ),
            Err(err) => err,
        };

    let gate_error = but_api::legacy::forge::classify_error(&self_assign_err)
        .context("T-LPR-043: self-assignment rejection must surface a structured denial")?;
    assert_eq!(
        gate_error.code,
        but_authz::Denial::PERM_DENIED_CODE,
        "T-LPR-043: self-assignment must be perm.denied"
    );

    // NO `(feat, auth)` assignment row may exist — the drive layer must not
    // narrate a self-assigned reviewer as independently reviewed.
    let rows = ctx
        .db
        .get_cache()?
        .local_review_assignments()
        .list_by_target(FEAT_REF)?;
    assert!(
        !rows.iter().any(|r| r.reviewer_principal == "auth"),
        "T-LPR-043: a self-assignment row for `auth` must NOT be persisted — R22 (got {rows:?})"
    );

    Ok(())
}

/// T-LPR-044 — an unauthorized self-resolve cannot suppress another party's
/// remediation signal. `auth` posts a `changes_requested`-style thread `t1`;
/// `other` (a non-author / non-assigned / non-`reviews:write` principal)
/// attempts `resolve_thread(t1, resolved=true)` and must be REJECTED, with
/// thread `t1` left unresolved. R22 narrows cross-principal forgery only.
#[tokio::test]
#[serial_test::serial]
async fn t_lpr_044_unauthorized_self_resolve_cannot_suppress_signal() -> anyhow::Result<()> {
    let (repo, _tmp) = lpr_governed_repo();
    let ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache();

    // `auth` posts a changes_requested-style thread t1 (the remediation
    // signal that another party will try to suppress). `auth` holds
    // comments:write so the post succeeds.
    temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("auth"))], async {
        but_api::legacy::forge::post_comment(
            ctx.to_sync(),
            "feat".to_owned(),
            "please fix".to_owned(),
            Some("f.rs".to_owned()),
            Some(12),
            "t1".to_owned(),
        )
        .await
    })
    .await
    .context("auth must post thread t1 as the remediation signal")?;

    // `other` holds comments:write but is NOT the thread author, NOT the
    // assigned reviewer, and NOT a reviews:write holder → resolve_thread
    // must be REJECTED.
    let resolve_err =
        match temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("other"))], async {
            but_api::legacy::forge::resolve_thread(
                ctx.to_sync(),
                "feat".to_owned(),
                "t1".to_owned(),
                true,
            )
            .await
        })
        .await
        {
            Ok(()) => bail!(
                "T-LPR-044: `other` (non-author/non-assigned/non-reviews:write) must NOT be \
             permitted to resolve_thread(t1) — R22 resolver-identity"
            ),
            Err(err) => err,
        };

    // The rejection must name the resolver-identity constraint (thread author
    // / assigned reviewer / reviews:write).
    let err_text = resolve_err.to_string();
    assert!(
        err_text.contains("thread author")
            || err_text.contains("assigned reviewer")
            || err_text.contains("reviews:write"),
        "T-LPR-044: the R22 denial must explain the resolver-identity constraint, got: {err_text}"
    );

    // Thread t1 must STAY UNRESOLVED — the unauthorized resolve must flip no
    // comment row. This is the load-bearing assertion: a cross-principal
    // actor cannot forge an all-clear and suppress the remediation signal.
    let t1_comments = ctx
        .db
        .get_cache()?
        .local_review_comments()
        .list_by_thread("feat", "t1")?;
    assert!(
        !t1_comments.is_empty(),
        "T-LPR-044 fixture: thread t1 must carry the seeded remediation comment"
    );
    assert!(
        t1_comments.iter().all(|c| !c.resolved),
        "T-LPR-044: thread t1 must STAY unresolved after the unauthorized resolve attempt — \
         R22 (got resolved count: {})",
        t1_comments.iter().filter(|c| c.resolved).count()
    );

    Ok(())
}

// ============================================================================
// Fixture + grep helpers (mirror invariant_build_gates.rs discipline)
// ============================================================================

/// A governed repo for the LPR-010 drive-layer proofs. Principals:
/// - `auth` — pull_requests:write + comments:write (the opener / thread author; the self-assignment target for T-LPR-043).
/// - `rev` — reviews:write + comments:write (the assign authority for T-LPR-043).
/// - `other` — comments:write ONLY (neither thread author nor assigned reviewer nor reviews:write holder — the unauthorized resolver for T-LPR-044).
fn lpr_governed_repo() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "auth"
permissions = ["contents:write", "pull_requests:write", "comments:write"]

[[principal]]
id = "rev"
permissions = ["reviews:write", "comments:write", "contents:read"]

[[principal]]
id = "other"
permissions = ["comments:write", "contents:read"]
EOF

cat >.gitbutler/gates.toml <<'EOF'
EOF

git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "governance config"
git checkout -b feat
echo feat-base >feat-base.txt
git add feat-base.txt
git commit -m "feat base"
git checkout main
"#,
        &repo,
    );
    (repo, tmp)
}

fn workspace_root() -> anyhow::Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("CARGO_MANIFEST_DIR must resolve from crates/but-api to the workspace root")
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

fn command_output(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    format!(
        "status: {}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        output.status
    )
}
